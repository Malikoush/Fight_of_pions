//! Le serveur de bout en bout, sur une vraie WebSocket : des clients qui ne connaissent
//! aucune règle et jouent les coups que le serveur leur propose. Chaque serveur a sa base,
//! en mémoire.

use fight_of_pions::{Action, Attack, Decision, DraftStep, Event, Player, PlayerView};
use futures_util::{SinkExt, StreamExt};
use serveur::{ClientMessage, Database, Opponent, ServerMessage};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

/// Lance un serveur sur un port libre ; rend l'adresse de sa WebSocket.
async fn start_server() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let database = Database::in_memory().await.unwrap();
    tokio::spawn(async move { axum::serve(listener, serveur::app(database)).await.unwrap() });
    format!("ws://{address}/ws")
}

struct Client {
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl Client {
    async fn connect(url: &str) -> Client {
        let (socket, _) = connect_async(url).await.unwrap();
        Client { socket }
    }

    /// Un client connecté à un compte neuf, qui a reçu son compte et sa collection.
    async fn registered(url: &str) -> Client {
        let mut client = Client::connect(url).await;
        client.send(ClientMessage::Register { pseudo: None }).await;
        client.account().await;
        client.collection().await;
        client
    }

    /// Le prochain message, qui doit être un compte : son numéro, son pseudo, son jeton.
    async fn account(&mut self) -> (i64, String, Option<String>) {
        match self.receive().await {
            ServerMessage::Account { id, pseudo, token } => (id, pseudo, token),
            other => panic!("un compte était attendu, reçu : {other:?}"),
        }
    }

    /// Le prochain message, qui doit être une collection : ses cartes et son trio.
    async fn collection(&mut self) -> (Vec<Attack>, Vec<Attack>) {
        match self.receive().await {
            ServerMessage::Collection { cards, cache_trio } => (cards, cache_trio),
            other => panic!("une collection était attendue, reçu : {other:?}"),
        }
    }

    /// Le prochain message, qui doit être une erreur : sa sorte.
    async fn error_kind(&mut self) -> String {
        match self.receive().await {
            ServerMessage::Error { kind, .. } => kind,
            other => panic!("une erreur était attendue, reçu : {other:?}"),
        }
    }

    async fn send(&mut self, message: ClientMessage) {
        let text = serde_json::to_string(&message).unwrap();
        self.socket.send(Message::Text(text.into())).await.unwrap();
    }

    /// Le prochain message, sauf les nouvelles de la présence de l'adversaire, que seuls les
    /// tests de la salle d'attente regardent (`receive_any`).
    async fn receive(&mut self) -> ServerMessage {
        loop {
            match self.receive_any().await {
                ServerMessage::Opponent { .. } => continue,
                message => return message,
            }
        }
    }

    async fn receive_any(&mut self) -> ServerMessage {
        loop {
            match self.socket.next().await.unwrap().unwrap() {
                Message::Text(text) => return serde_json::from_str(text.as_str()).unwrap(),
                _ => continue,
            }
        }
    }

    /// Le prochain message, qui doit être une place.
    async fn seated(&mut self) -> (String, Player, String) {
        match self.receive().await {
            ServerMessage::Seated { code, player, key } => (code, player, key),
            other => panic!("une place était attendue, reçu : {other:?}"),
        }
    }

    /// Le prochain message, qui doit être une vue.
    async fn state(&mut self) -> (PlayerView, Vec<Event>) {
        match self.receive().await {
            ServerMessage::State { view, events } => (*view, events),
            other => panic!("une vue était attendue, reçu : {other:?}"),
        }
    }

    /// Le prochain message, qui doit être une erreur.
    async fn error(&mut self) -> String {
        match self.receive().await {
            ServerMessage::Error { message, .. } => message,
            other => panic!("une erreur était attendue, reçu : {other:?}"),
        }
    }

    async fn play(&mut self, action: Action) {
        self.send(ClientMessage::Play { action }).await;
    }
}

/// Le coup qu'un client sans règle choisit : passer ou finir son tour s'il le peut, sinon
/// le premier coup permis (une carte au draft, une pose…).
fn simple_choice(view: &PlayerView) -> Action {
    view.legal_actions
        .iter()
        .copied()
        .find(|action| matches!(action, Action::Pass | Action::EndTurn))
        .unwrap_or(view.legal_actions[0])
}

#[tokio::test]
async fn two_humans_play_and_each_sees_only_its_own_cache() {
    let url = start_server().await;
    let mut one = Client::registered(&url).await;
    one.send(ClientMessage::Create {
        opponent: Opponent::Human,
        variant: None,
    })
    .await;
    let (code, player, _) = one.seated().await;
    assert_eq!(player, Player::One);
    let (mut view_of_one, _) = one.state().await;
    assert_eq!(
        view_of_one.decision,
        Some(Decision::Draft {
            step: DraftStep::Ban(Player::One)
        })
    );

    let mut two = Client::registered(&url).await;
    two.send(ClientMessage::Join {
        code: code.to_lowercase(),
    })
    .await;
    assert_eq!(two.seated().await.1, Player::Two);
    let (mut view_of_two, _) = two.state().await;
    assert!(view_of_two.legal_actions.is_empty(), "ce n'est pas à J2");

    // Les bans et les picks, publics, jusqu'à la cache de J1 (choisie en dernier).
    loop {
        let step = match view_of_one.decision {
            Some(Decision::Draft {
                step: DraftStep::Cache(_),
            }) => break,
            Some(Decision::Draft { step }) => step,
            other => panic!("un choix de draft était attendu, reçu : {other:?}"),
        };
        if step.player() == Player::One {
            one.play(simple_choice(&view_of_one)).await;
        } else {
            two.play(simple_choice(&view_of_two)).await;
        }
        view_of_one = one.state().await.0;
        view_of_two = two.state().await.0;
    }

    // J1 met une carte en cache : J2 voit qu'il y a une carte, pas laquelle.
    let cache = match simple_choice(&view_of_one) {
        Action::Draft(card) => card,
        other => panic!("un choix de draft était attendu, reçu : {other:?}"),
    };
    // Un aperçu ne montre pas la suite d'un coup qui n'est pas un choix de carte.
    one.send(ClientMessage::Preview {
        actions: vec![Action::Draft(cache)],
    })
    .await;
    assert_eq!(one.receive().await, ServerMessage::Preview { view: None });
    one.play(Action::Draft(cache)).await;
    let (seen_by_one, events_one) = one.state().await;
    let (seen_by_two, events_two) = two.state().await;
    // La cache est la dernière carte de l'arsenal, après les picks.
    let cached_slot = seen_by_one.sides[0].arsenal.last().copied().flatten();
    assert_eq!(cached_slot.map(|slot| slot.attack()), Some(cache));
    assert_eq!(seen_by_two.sides[0].arsenal.last(), Some(&None));
    assert_eq!(events_one.len(), 1);
    // J2 apprend qu'une carte est en cache, sans la carte.
    assert_eq!(
        events_two,
        [Event::CardCached {
            player: Player::One
        }]
    );
}

#[tokio::test]
async fn requests_out_of_turn_or_out_of_a_room_are_refused() {
    let url = start_server().await;
    let mut client = Client::registered(&url).await;
    client.play(Action::EndTurn).await;
    assert!(client.error().await.contains("salle"));
    client
        .send(ClientMessage::Join {
            code: "ZZZZZ".to_string(),
        })
        .await;
    assert!(client.error().await.contains("ZZZZZ"));
    client
        .socket
        .send(Message::Text("bonjour".into()))
        .await
        .unwrap();
    assert!(client.error().await.contains("illisible"));

    client
        .send(ClientMessage::Create {
            opponent: Opponent::Human,
            variant: None,
        })
        .await;
    let (code, _, _) = client.seated().await;
    client.state().await;
    let mut other = Client::registered(&url).await;
    other.send(ClientMessage::Join { code }).await;
    other.seated().await;
    other.state().await;
    other.play(Action::EndTurn).await;
    assert_eq!(other.error().await, "ce n'est pas à toi de jouer");
}

#[tokio::test]
#[ignore = "fait jouer un bot une partie entière : à lancer à la main"]
async fn a_whole_game_against_a_server_bot() {
    let url = start_server().await;
    let mut client = Client::registered(&url).await;
    client
        .send(ClientMessage::Create {
            opponent: Opponent::Random,
            variant: None,
        })
        .await;
    client.seated().await;
    let mut view = client.state().await.0;
    let mut messages = 0;
    while view.end.is_none() {
        if !view.legal_actions.is_empty() {
            client.play(simple_choice(&view)).await;
        }
        view = client.state().await.0;
        messages += 1;
        assert!(messages < 5000, "partie trop longue");
    }
}

#[tokio::test]
async fn a_player_gets_its_seat_back_with_its_key() {
    let url = start_server().await;
    let mut client = Client::registered(&url).await;
    client
        .send(ClientMessage::Create {
            opponent: Opponent::Human,
            variant: None,
        })
        .await;
    let (code, player, key) = client.seated().await;
    let (before, _) = client.state().await;
    drop(client);

    let mut back = Client::connect(&url).await;
    back.send(ClientMessage::Rejoin {
        code: code.clone(),
        player,
        key: "faux".to_string(),
    })
    .await;
    assert!(back.error().await.contains("clé"));
    back.send(ClientMessage::Rejoin { code, player, key }).await;
    back.seated().await;
    assert_eq!(back.state().await.0, before);
    // Une vue perdue se redemande.
    back.send(ClientMessage::Sync).await;
    assert_eq!(back.state().await, (before, Vec::new()));
}

#[tokio::test]
async fn the_card_catalog_is_sent_on_request_even_without_a_seat() {
    let url = start_server().await;
    let mut client = Client::registered(&url).await;
    client.send(ClientMessage::Cards).await;
    let cards = match client.receive().await {
        ServerMessage::Cards { cards } => cards,
        other => panic!("le catalogue était attendu, reçu : {other:?}"),
    };
    let bear = cards
        .iter()
        .find(|family| family.kind == fight_of_pions::AttackKind::Ours)
        .expect("l'Ours est au catalogue");
    assert_eq!(bear.name, "Ours");
    assert!(!bear.rarities.is_empty());
    assert!(cards.iter().any(|family| family.reaction));
    // Le catalogue ne demande pas de place : on peut encore créer une salle ensuite, puis le
    // redemander une fois assis.
    client
        .send(ClientMessage::Create {
            opponent: Opponent::Human,
            variant: None,
        })
        .await;
    client.seated().await;
    client.state().await;
    client.send(ClientMessage::Cards).await;
    assert!(matches!(
        client.receive().await,
        ServerMessage::Cards { cards: again } if again == cards
    ));
}

#[tokio::test]
async fn each_player_learns_when_the_other_arrives_and_leaves() {
    let url = start_server().await;
    let mut one = Client::registered(&url).await;
    one.send(ClientMessage::Create {
        opponent: Opponent::Human,
        variant: None,
    })
    .await;
    let (code, _, _) = one.seated().await;
    one.state().await;

    let mut two = Client::registered(&url).await;
    two.send(ClientMessage::Join { code }).await;
    two.seated().await;
    assert!(matches!(
        two.receive_any().await,
        ServerMessage::State { .. }
    ));
    assert_eq!(
        two.receive_any().await,
        ServerMessage::Opponent { connected: true }
    );
    assert_eq!(
        one.receive_any().await,
        ServerMessage::Opponent { connected: true }
    );

    drop(two);
    assert_eq!(
        one.receive_any().await,
        ServerMessage::Opponent { connected: false }
    );
}

/// Une partie entre deux humains, jouée au plus simple jusqu'à la cache de J1 (choisie en
/// dernier) ; rend les deux clients et la vue de chacun.
async fn draft_until_the_first_cache(
    mut one: Client,
    mut two: Client,
) -> (Client, Client, PlayerView, PlayerView) {
    one.send(ClientMessage::Create {
        opponent: Opponent::Human,
        variant: None,
    })
    .await;
    let (code, _, _) = one.seated().await;
    let (mut view_of_one, _) = one.state().await;
    two.send(ClientMessage::Join { code }).await;
    two.seated().await;
    let (mut view_of_two, _) = two.state().await;
    loop {
        let step = match view_of_one.decision {
            Some(Decision::Draft {
                step: DraftStep::Cache(_),
            }) => break,
            Some(Decision::Draft { step }) => step,
            other => panic!("un choix de draft était attendu, reçu : {other:?}"),
        };
        if step.player() == Player::One {
            one.play(simple_choice(&view_of_one)).await;
        } else {
            two.play(simple_choice(&view_of_two)).await;
        }
        view_of_one = one.state().await.0;
        view_of_two = two.state().await.0;
    }
    (one, two, view_of_one, view_of_two)
}

/// Les cartes que ce joueur peut choisir au draft, triées.
fn draft_choices(view: &PlayerView) -> Vec<Attack> {
    let mut cards: Vec<Attack> = view
        .legal_actions
        .iter()
        .map(|action| match action {
            Action::Draft(card) => *card,
            other => panic!("un choix de draft était attendu, reçu : {other:?}"),
        })
        .collect();
    cards.sort_unstable();
    cards
}

#[tokio::test]
async fn an_account_is_created_once_then_found_again_with_its_token() {
    let url = start_server().await;
    let mut first = Client::connect(&url).await;
    first
        .send(ClientMessage::Register {
            pseudo: Some("  Martin ".to_string()),
        })
        .await;
    let (id, pseudo, token) = first.account().await;
    assert_eq!(pseudo, "Martin");
    let token = token.expect("le jeton est donné à la création");
    let (cards, trio) = first.collection().await;
    assert!(!cards.is_empty(), "le paquet de départ");
    assert!(trio.is_empty());
    first.send(ClientMessage::Register { pseudo: None }).await;
    assert_eq!(first.error_kind().await, "already_logged_in");
    drop(first);

    let mut again = Client::connect(&url).await;
    again
        .send(ClientMessage::Login {
            token: "faux".to_string(),
        })
        .await;
    assert_eq!(again.error_kind().await, "unknown_account");
    again.send(ClientMessage::Login { token }).await;
    assert_eq!(again.account().await, (id, "Martin".to_string(), None));
    assert_eq!(again.collection().await, (cards, trio));

    again
        .send(ClientMessage::SetPseudo {
            pseudo: "<b>".to_string(),
        })
        .await;
    assert_eq!(again.error_kind().await, "bad_pseudo");
    again
        .send(ClientMessage::SetPseudo {
            pseudo: "Martin le Pion".to_string(),
        })
        .await;
    assert_eq!(
        again.account().await,
        (id, "Martin le Pion".to_string(), None)
    );
}

#[tokio::test]
async fn a_room_needs_an_account() {
    let url = start_server().await;
    let mut guest = Client::connect(&url).await;
    guest
        .send(ClientMessage::Create {
            opponent: Opponent::Human,
            variant: None,
        })
        .await;
    assert_eq!(guest.error_kind().await, "not_logged_in");
    guest.send(ClientMessage::Collection).await;
    assert_eq!(guest.error_kind().await, "not_logged_in");
    guest
        .send(ClientMessage::Join {
            code: "ZZZZZ".to_string(),
        })
        .await;
    assert_eq!(guest.error_kind().await, "not_logged_in");
}

#[tokio::test]
async fn a_trio_holds_up_to_three_owned_cards() {
    let url = start_server().await;
    let mut client = Client::connect(&url).await;
    client.send(ClientMessage::Register { pseudo: None }).await;
    client.account().await;
    let (cards, _) = client.collection().await;
    let legendary = Attack {
        kind: fight_of_pions::AttackKind::Ours,
        rarity: fight_of_pions::Rarity::Legendary,
    };
    assert!(!cards.contains(&legendary));
    client
        .send(ClientMessage::SetCacheTrio {
            cards: vec![cards[0], legendary],
        })
        .await;
    assert_eq!(client.error_kind().await, "not_owned");
    client
        .send(ClientMessage::SetCacheTrio {
            cards: cards[..4].to_vec(),
        })
        .await;
    assert_eq!(client.error_kind().await, "bad_cache_trio");
    let trio = vec![cards[2], cards[0]];
    client
        .send(ClientMessage::SetCacheTrio {
            cards: trio.clone(),
        })
        .await;
    assert_eq!(client.collection().await, (cards.clone(), trio.clone()));
    client.send(ClientMessage::Collection).await;
    assert_eq!(client.collection().await, (cards, trio));
}

#[tokio::test]
async fn each_player_caches_one_of_its_own_cards() {
    let url = start_server().await;
    // J1 a rangé un trio ; J2 non : le serveur lui tire 3 de ses cartes au hasard.
    let mut one = Client::connect(&url).await;
    one.send(ClientMessage::Register { pseudo: None }).await;
    one.account().await;
    let (cards, _) = one.collection().await;
    let trio = vec![cards[5], cards[1]];
    one.send(ClientMessage::SetCacheTrio {
        cards: trio.clone(),
    })
    .await;
    one.collection().await;
    let mut two = Client::connect(&url).await;
    two.send(ClientMessage::Register { pseudo: None }).await;
    two.account().await;
    let (cards_of_two, _) = two.collection().await;

    let (mut one, mut two, view_of_one, _) = draft_until_the_first_cache(one, two).await;
    assert_eq!(draft_choices(&view_of_one), [cards[1], cards[5]]);
    let not_in_trio = Action::Draft(cards[0]);
    one.play(not_in_trio).await;
    assert!(one.error().await.contains("draft"));
    one.play(Action::Draft(cards[5])).await;
    one.state().await;
    let (view_of_two, _) = two.state().await;
    let choices_of_two = draft_choices(&view_of_two);
    assert_eq!(choices_of_two.len(), 3);
    assert!(
        choices_of_two
            .iter()
            .all(|card| cards_of_two.contains(card))
    );
}

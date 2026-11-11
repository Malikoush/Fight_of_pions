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
    tokio::spawn(async move {
        axum::serve(listener, serveur::app(database, None))
            .await
            .unwrap()
    });
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
        Client::named(url, None).await.0
    }

    /// Un client connecté à un compte neuf de ce pseudo, qui a reçu son compte et sa
    /// collection ; son jeton.
    async fn named(url: &str, pseudo: Option<&str>) -> (Client, String) {
        let mut client = Client::connect(url).await;
        client
            .send(ClientMessage::Register {
                pseudo: pseudo.map(str::to_string),
            })
            .await;
        let (_, _, token) = client.account().await;
        client.collection().await;
        (client, token.unwrap())
    }

    /// Un client qui revient sur ce compte avec son jeton : rend vrai s'il est rassis dans
    /// sa salle (sa place, puis sa vue, arrivent alors avant sa collection).
    async fn logged_in(url: &str, token: &str) -> (Client, bool) {
        let mut client = Client::connect(url).await;
        client
            .send(ClientMessage::Login {
                token: token.to_string(),
            })
            .await;
        let resumed = match client.receive().await {
            ServerMessage::Account { resumed, .. } => resumed,
            other => panic!("un compte était attendu, reçu : {other:?}"),
        };
        (client, resumed)
    }

    /// Le prochain message, qui doit être un compte : son numéro, son pseudo, son jeton.
    async fn account(&mut self) -> (i64, String, Option<String>) {
        match self.receive().await {
            ServerMessage::Account {
                id, pseudo, token, ..
            } => (id, pseudo, token),
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
            ServerMessage::Seated {
                code, player, key, ..
            } => (code, player, key),
            other => panic!("une place était attendue, reçu : {other:?}"),
        }
    }

    /// Le prochain message, qui doit être l'annonce du lancement de la partie.
    async fn launched(&mut self) {
        match self.receive().await {
            ServerMessage::Launched => {}
            other => panic!("le lancement était attendu, reçu : {other:?}"),
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

/// Une salle entre amis : `host` la crée (à une place tirée au sort), `guest` la rejoint
/// (l'autre place), puis `host` la lance. Rend les deux clients rangés par place (J1, J2),
/// la première vue de chacun, et vrai si l'hôte est J1.
async fn friends_room(
    mut host: Client,
    mut guest: Client,
) -> (Client, Client, PlayerView, PlayerView, bool) {
    host.send(ClientMessage::Create {
        opponent: Opponent::Human,
        variant: None,
    })
    .await;
    let (code, host_seat, _) = host.seated().await;
    let (host_view, _) = host.state().await;
    guest
        .send(ClientMessage::Join {
            code: code.to_lowercase(),
        })
        .await;
    let (_, guest_seat, _) = guest.seated().await;
    assert_ne!(guest_seat, host_seat, "la place laissée libre");
    let (guest_view, _) = guest.state().await;
    // Personne ne joue avant le lancement.
    host.play(Action::EndTurn).await;
    assert_eq!(host.error_kind().await, "not_launched");
    guest.send(ClientMessage::Launch).await;
    assert_eq!(guest.error_kind().await, "not_host");
    host.send(ClientMessage::Launch).await;
    host.launched().await;
    guest.launched().await;
    if host_seat == Player::One {
        (host, guest, host_view, guest_view, true)
    } else {
        (guest, host, guest_view, host_view, false)
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
    let host = Client::registered(&url).await;
    let guest = Client::registered(&url).await;
    let (mut one, mut two, mut view_of_one, mut view_of_two, _) = friends_room(host, guest).await;
    assert_eq!(
        view_of_one.decision,
        Some(Decision::Draft {
            step: DraftStep::Ban(Player::One)
        })
    );
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

    client.send(ClientMessage::Launch).await;
    assert!(client.error().await.contains("salle"));
    let other = Client::registered(&url).await;
    let (_, mut second, _, _, _) = friends_room(client, other).await;
    second.play(Action::EndTurn).await;
    assert_eq!(second.error().await, "ce n'est pas à toi de jouer");
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
async fn a_player_is_seated_again_when_logging_back_in() {
    let url = start_server().await;
    let (mut client, token) = Client::named(&url, None).await;
    client
        .send(ClientMessage::Create {
            opponent: Opponent::Human,
            variant: None,
        })
        .await;
    let (code, player, key) = client.seated().await;
    let (before, _) = client.state().await;
    drop(client);

    let (mut back, resumed) = Client::logged_in(&url, &token).await;
    assert!(resumed);
    assert_eq!(back.seated().await, (code.clone(), player, key.clone()));
    assert_eq!(back.state().await.0, before);
    back.collection().await;
    // Une vue perdue se redemande.
    back.send(ClientMessage::Sync).await;
    assert_eq!(back.state().await, (before, Vec::new()));

    // Sa clé ne sert à aucun autre compte.
    let mut other = Client::registered(&url).await;
    other
        .send(ClientMessage::Rejoin { code, player, key })
        .await;
    assert_eq!(other.error_kind().await, "bad_key");
}

#[tokio::test]
async fn a_new_connection_of_the_account_takes_its_seat() {
    let url = start_server().await;
    let (mut first, token) = Client::named(&url, None).await;
    first
        .send(ClientMessage::Create {
            opponent: Opponent::Human,
            variant: None,
        })
        .await;
    first.seated().await;
    first.state().await;

    let (mut second, resumed) = Client::logged_in(&url, &token).await;
    assert!(resumed);
    second.seated().await;
    assert_eq!(first.error_kind().await, "seat_taken");
    second.state().await;
    second.collection().await;
    // Un compte, une salle : il faut quitter celle-ci pour en ouvrir une autre.
    second
        .send(ClientMessage::Create {
            opponent: Opponent::Random,
            variant: None,
        })
        .await;
    assert_eq!(second.error_kind().await, "already_seated");
    // L'ancienne connexion n'est plus assise, mais son compte l'est toujours.
    first
        .send(ClientMessage::Create {
            opponent: Opponent::Random,
            variant: None,
        })
        .await;
    assert_eq!(first.error_kind().await, "already_seated");
}

#[tokio::test]
async fn leaving_a_launched_game_loses_it_by_forfeit() {
    let url = start_server().await;
    let host = Client::registered(&url).await;
    let guest = Client::registered(&url).await;
    let (mut one, mut two, _, _, _) = friends_room(host, guest).await;

    one.send(ClientMessage::Leave).await;
    assert_eq!(one.receive().await, ServerMessage::Left);
    let (view, events) = two.state().await;
    assert_eq!(
        view.end.map(|end| (end.winner(), end.reason)),
        Some((Some(Player::Two), fight_of_pions::EndReason::Forfeit))
    );
    assert!(matches!(events.last(), Some(Event::GameEnded(_))));

    // Libre, il peut ouvrir une autre salle sur la même connexion.
    one.send(ClientMessage::Create {
        opponent: Opponent::Human,
        variant: None,
    })
    .await;
    one.seated().await;
    one.state().await;
    one.send(ClientMessage::Leave).await;
    assert_eq!(one.receive().await, ServerMessage::Left);
    one.send(ClientMessage::Leave).await;
    assert_eq!(one.receive().await, ServerMessage::Left, "même sans salle");
}

#[tokio::test]
async fn a_host_who_leaves_before_the_launch_closes_the_room() {
    let url = start_server().await;
    let mut host = Client::registered(&url).await;
    host.send(ClientMessage::Create {
        opponent: Opponent::Human,
        variant: None,
    })
    .await;
    let (code, _, _) = host.seated().await;
    host.state().await;
    let mut guest = Client::registered(&url).await;
    guest.send(ClientMessage::Join { code: code.clone() }).await;
    guest.seated().await;
    guest.state().await;

    host.send(ClientMessage::Leave).await;
    assert_eq!(host.receive().await, ServerMessage::Left);
    assert_eq!(guest.error_kind().await, "room_closed");
    let mut late = Client::registered(&url).await;
    late.send(ClientMessage::Join { code }).await;
    assert_eq!(late.error_kind().await, "unknown_room");
}

#[tokio::test]
async fn a_stopping_server_closes_its_connections() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = serveur::Server::new(Database::in_memory().await.unwrap(), None);
    let router = server.router();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let mut client = Client::registered(&format!("ws://{address}/ws")).await;

    server.shutdown().await;
    loop {
        match client.socket.next().await {
            Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
            Some(Ok(_)) => continue,
        }
    }
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
    let (mut one, _) = Client::named(&url, Some("Ana")).await;
    one.send(ClientMessage::Create {
        opponent: Opponent::Human,
        variant: None,
    })
    .await;
    let (code, _, _) = one.seated().await;
    one.state().await;

    let (mut two, _) = Client::named(&url, Some("Bob")).await;
    two.send(ClientMessage::Join { code }).await;
    two.seated().await;
    assert!(matches!(
        two.receive_any().await,
        ServerMessage::State { .. }
    ));
    let presence = |connected, pseudo: &str| ServerMessage::Opponent {
        connected,
        pseudo: pseudo.to_string(),
    };
    assert_eq!(two.receive_any().await, presence(true, "Ana"));
    assert_eq!(one.receive_any().await, presence(true, "Bob"));

    drop(two);
    assert_eq!(one.receive_any().await, presence(false, "Bob"));
}

/// Une partie entre deux humains (`host` crée la salle, `guest` la rejoint), jouée au plus
/// simple jusqu'à la cache de J1 (choisie en dernier) ; rend les deux clients rangés par
/// place (J1, J2), la vue de chacun, et vrai si l'hôte est J1.
async fn draft_until_the_first_cache(
    host: Client,
    guest: Client,
) -> (Client, Client, PlayerView, PlayerView, bool) {
    let (mut one, mut two, mut view_of_one, mut view_of_two, host_is_one) =
        friends_room(host, guest).await;
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
    (one, two, view_of_one, view_of_two, host_is_one)
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
    // L'hôte a rangé un trio ; son invité non : le serveur lui tire 3 de ses cartes au hasard.
    let mut host = Client::connect(&url).await;
    host.send(ClientMessage::Register { pseudo: None }).await;
    host.account().await;
    let (cards_of_host, _) = host.collection().await;
    let mut trio = vec![cards_of_host[5], cards_of_host[1]];
    host.send(ClientMessage::SetCacheTrio {
        cards: trio.clone(),
    })
    .await;
    host.collection().await;
    trio.sort_unstable();
    let mut guest = Client::connect(&url).await;
    guest.send(ClientMessage::Register { pseudo: None }).await;
    guest.account().await;
    let (cards_of_guest, _) = guest.collection().await;
    // Les choix de cache de l'hôte, puis de son invité.
    let check = |choices: &[Attack], is_host: bool| {
        if is_host {
            assert_eq!(choices, trio.as_slice(), "le trio de l'hôte");
        } else {
            assert_eq!(choices.len(), 3);
            assert!(choices.iter().all(|card| cards_of_guest.contains(card)));
        }
    };
    let owned_by = |is_host: bool| {
        if is_host {
            &cards_of_host
        } else {
            &cards_of_guest
        }
    };

    // J1 choisit sa cache le premier, puis J2.
    let (mut one, mut two, view_of_one, _, host_is_one) =
        draft_until_the_first_cache(host, guest).await;
    let choices = draft_choices(&view_of_one);
    check(&choices, host_is_one);
    let outside = owned_by(host_is_one)
        .iter()
        .find(|card| !choices.contains(card))
        .copied()
        .unwrap();
    one.play(Action::Draft(outside)).await;
    assert!(one.error().await.contains("draft"));
    one.play(Action::Draft(choices[0])).await;
    one.state().await;
    let (view_of_two, _) = two.state().await;
    check(&draft_choices(&view_of_two), !host_is_one);
}

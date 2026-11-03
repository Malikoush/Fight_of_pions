//! Le serveur de bout en bout, sur une vraie WebSocket : des clients qui ne connaissent
//! aucune règle et jouent les coups que le serveur leur propose.

use fight_of_pions::{Action, Decision, DraftStep, Event, Player, PlayerView};
use futures_util::{SinkExt, StreamExt};
use serveur::{ClientMessage, Opponent, ServerMessage};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

/// Lance un serveur sur un port libre ; rend l'adresse de sa WebSocket.
async fn start_server() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, serveur::app()).await.unwrap() });
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

    async fn send(&mut self, message: ClientMessage) {
        let text = serde_json::to_string(&message).unwrap();
        self.socket.send(Message::Text(text.into())).await.unwrap();
    }

    async fn receive(&mut self) -> ServerMessage {
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
                code,
                player,
                token,
            } => (code, player, token),
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
            ServerMessage::Error { message } => message,
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
    let mut one = Client::connect(&url).await;
    one.send(ClientMessage::Create {
        opponent: Opponent::Human,
        variant: None,
    })
    .await;
    let (code, player, _) = one.seated().await;
    assert_eq!(player, Player::One);
    let (view_of_one, _) = one.state().await;
    assert_eq!(
        view_of_one.decision,
        Some(Decision::Draft {
            step: DraftStep::Cache(Player::One)
        })
    );

    let mut two = Client::connect(&url).await;
    two.send(ClientMessage::Join {
        code: code.to_lowercase(),
    })
    .await;
    assert_eq!(two.seated().await.1, Player::Two);
    let (view, _) = two.state().await;
    assert!(view.legal_actions.is_empty(), "ce n'est pas à J2");

    // J1 met une carte en cache : J2 voit qu'il y a une carte, pas laquelle.
    let cache = match simple_choice(&view_of_one) {
        Action::Draft(card) => card,
        other => panic!("un choix de draft était attendu, reçu : {other:?}"),
    };
    one.play(Action::Draft(cache)).await;
    let (seen_by_one, events_one) = one.state().await;
    let (seen_by_two, events_two) = two.state().await;
    assert_eq!(seen_by_one.sides[0].arsenal.len(), 1);
    assert_eq!(seen_by_one.sides[0].arsenal[0].unwrap().attack(), cache);
    assert_eq!(seen_by_two.sides[0].arsenal, [None]);
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
    let mut client = Client::connect(&url).await;
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
    let mut other = Client::connect(&url).await;
    other.send(ClientMessage::Join { code }).await;
    other.seated().await;
    other.state().await;
    other.play(Action::EndTurn).await;
    assert_eq!(other.error().await, "ce n'est pas à toi de jouer");
}

#[tokio::test]
async fn a_whole_game_against_a_server_bot() {
    let url = start_server().await;
    let mut client = Client::connect(&url).await;
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
async fn a_player_gets_its_seat_back_with_its_token() {
    let url = start_server().await;
    let mut client = Client::connect(&url).await;
    client
        .send(ClientMessage::Create {
            opponent: Opponent::Human,
            variant: None,
        })
        .await;
    let (code, player, token) = client.seated().await;
    let (before, _) = client.state().await;
    drop(client);

    let mut back = Client::connect(&url).await;
    back.send(ClientMessage::Rejoin {
        code: code.clone(),
        player,
        token: "faux".to_string(),
    })
    .await;
    assert!(back.error().await.contains("jeton"));
    back.send(ClientMessage::Rejoin {
        code,
        player,
        token,
    })
    .await;
    back.seated().await;
    assert_eq!(back.state().await.0, before);
}

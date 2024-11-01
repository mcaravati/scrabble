use crate::events::Event;
use crate::events::Event::Lobby;
use crate::manager::Manager;
use crate::response::Response;
use serde::{Deserialize, Serialize};
use socketioxide::extract::{AckSender, Data, SocketRef};
use tokio::sync::mpsc;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase", untagged)]
enum LobbyRequest {
    ListGames,
}

pub enum LobbyEvent {
    ListGames { ack_sender: AckSender },
}

async fn handle_list_games_request(
    message: LobbyRequest,
    ack_sender: AckSender,
    sender: mpsc::Sender<Event>,
) {
    if let LobbyRequest::ListGames = message {
        sender
            .send(Event::Lobby(LobbyEvent::ListGames { ack_sender }))
            .await
            .unwrap();
    }
}

pub fn on_connect(socket: SocketRef, sender: mpsc::Sender<Event>) {
    let sender_clone = sender.clone();

    socket.on(
        "list-games",
        |socket_ref: SocketRef, Data::<LobbyRequest>(message), ack_sender: AckSender| async move {
            handle_list_games_request(message, ack_sender, sender_clone).await;
        },
    );
}

pub fn handle_events(event: Event, manager: &mut Manager) {
    if let Lobby(event) = event {
        match event {
            // A player asked to see the games list
            LobbyEvent::ListGames { ack_sender } => {
                let response = Response::from_data(manager.get_game_list());

                ack_sender.send(&response).unwrap();
            }
        }
    }
}

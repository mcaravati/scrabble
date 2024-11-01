use crate::events::Event;
use crate::events::Event::Game;
use crate::manager::Manager;
use crate::player::Player;
use crate::response::Response;
use serde::{Deserialize, Serialize};
use socketioxide::extract::{AckSender, Data, SocketRef};
use tokio::sync::mpsc;
use tracing::debug;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase", untagged)]
enum GameRequest {
    Register { game_uuid: Uuid, username: String },
    Logout { game_uuid: Uuid, player_uuid: Uuid },
    Id { player_uuid: Uuid },
}

pub enum GameEvent {
    Registration {
        game_uuid: Uuid,
        player: Player,
        ack: AckSender,
    },
    Logout {
        game_uuid: Uuid,
        player_uuid: Uuid,
        ack: AckSender,
    },
    WhoAmI {
        socket_ref: SocketRef,
        player_uuid: Uuid,
        ack_sender: AckSender,
    },
}

async fn handle_registration_request(
    socket: SocketRef,
    data: GameRequest,
    ack: AckSender,
    sender: mpsc::Sender<Event>,
) {
    if let GameRequest::Register {
        game_uuid,
        username,
    } = data
    {
        if socket.extensions.get::<Player>().is_none() {
            let uuid = Uuid::new_v4();
            let player = Player::new(&uuid, &username);

            // Inform the manager there's a new player
            sender
                .send(Event::Game(GameEvent::Registration {
                    game_uuid,
                    player: player.clone(),
                    ack,
                }))
                .await
                .unwrap();

            // Associate the player to the socket for easy access
            socket.extensions.insert::<Player>(player);

            debug!(?uuid, %username, "Player connected");
        }
    }
}

async fn handle_logout_request(
    socket: SocketRef,
    data: GameRequest,
    ack: AckSender,
    sender: mpsc::Sender<Event>,
) {
    if let GameRequest::Logout {
        game_uuid,
        player_uuid,
    } = data
    {
        if socket.extensions.get::<Player>().is_some() {
            sender
                .send(Event::Game(GameEvent::Logout {
                    game_uuid,
                    player_uuid,
                    ack,
                }))
                .await
                .unwrap();

            socket.extensions.remove::<Player>().unwrap();
        }
    }
}

async fn handle_id_request(
    socket: SocketRef,
    data: GameRequest,
    ack_sender: AckSender,
    sender: mpsc::Sender<Event>,
) {
    if let GameRequest::Id { player_uuid } = data {
        if socket.extensions.get::<Player>().is_none() {
            sender
                .send(Event::Game(GameEvent::WhoAmI {
                    socket_ref: socket,
                    player_uuid,
                    ack_sender,
                }))
                .await
                .unwrap()
        }
    }
}

fn on_connect(socket: SocketRef, sender: mpsc::Sender<Event>) {
    let sender_clone_1 = sender.clone();
    let sender_clone_2 = sender.clone();
    let sender_clone_3 = sender.clone();

    socket.on(
        "register_request",
        |socket: SocketRef, Data::<GameRequest>(data), ack: AckSender| async move {
            handle_registration_request(socket, data, ack, sender_clone_1).await;
        },
    );

    socket.on(
        "logout",
        |socket: SocketRef, Data::<GameRequest>(data), ack: AckSender| async move {
            handle_logout_request(socket, data, ack, sender_clone_2).await;
        },
    );

    socket.on(
        "whoami",
        |socket: SocketRef, Data::<GameRequest>(message), ack: AckSender| async move {
            handle_id_request(socket, message, ack, sender_clone_3).await;
        },
    );
}

pub fn handle_events(event: Event, manager: &mut Manager) {
    if let Game(event) = event {
        match event {
            // A player found a game and decided to play
            GameEvent::Registration {
                game_uuid,
                player,
                ack,
            } => {
                let response = match manager.register_player_to_game(&game_uuid, player) {
                    Ok(player) => Response::from_data(player.get_id()),
                    Err(error) => Response::from_error(error),
                };

                ack.send(&response).ok();
            }

            // A player hit the `Log out` button
            GameEvent::Logout {
                game_uuid,
                player_uuid,
                ack,
            } => {
                let response = match manager.remove_player_from_game(&game_uuid, &player_uuid) {
                    Err(error) => Response::from_error(error),
                    Ok(_) => Response::from_data("Player successfully removed"),
                };

                ack.send(&response).unwrap();
            }

            // A player refreshed their page, flushing the data
            GameEvent::WhoAmI {
                socket_ref,
                player_uuid,
                ack_sender,
            } => {
                let response = match manager.player_from_uuid(&player_uuid) {
                    Ok(player) => {
                        socket_ref.extensions.insert::<Player>(player.clone());
                        Response::from_data(player)
                    }
                    Err(error) => Response::from_error(error),
                };

                ack_sender.send(&response).unwrap();
            }
        }
    }
}

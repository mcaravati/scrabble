use crate::events::Event;
use crate::events::Event::Game;
use crate::manager::Manager;
use crate::player::Player;
use crate::response::Response;
use serde::{Deserialize, Serialize};
use socketioxide::extract::{AckSender, Data, SocketRef};
use socketioxide::SocketIo;
use tokio::sync::mpsc;
use tracing::debug;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase", untagged)]
enum GameRequest {
    Register { game_uuid: Uuid, username: String },
    Logout { game_uuid: Uuid, player_uuid: Uuid },
    Id { player_uuid: Uuid },
    PlayerList,
    Start,
}

pub enum GameEvent {
    Registration {
        socket_ref: SocketRef,
        game_uuid: Uuid,
        player: Player,
        ack_sender: AckSender,
    },
    Logout {
        socket_ref: SocketRef,
        game_uuid: Uuid,
        player_uuid: Uuid,
        ack: AckSender,
    },
    WhoAmI {
        socket_ref: SocketRef,
        player_uuid: Uuid,
        ack_sender: AckSender,
    },
    PlayerList {
        game_uuid: Uuid,
        ack_sender: AckSender,
    },
    StartGame {
        game_uuid: Uuid,
    },
}

async fn handle_registration_request(
    socket_ref: SocketRef,
    data: GameRequest,
    ack_sender: AckSender,
    sender: mpsc::Sender<Event>,
) {
    if let GameRequest::Register {
        game_uuid,
        username,
    } = data
    {
        if socket_ref.extensions.get::<Player>().is_none() {
            let uuid = Uuid::new_v4();
            let player = Player::new(&uuid, &username);

            // Associate the player to the socket for easy access
            socket_ref.extensions.insert::<Player>(player.clone());

            // Inform the manager there's a new player
            sender
                .send(Event::Game(GameEvent::Registration {
                    socket_ref,
                    game_uuid,
                    player,
                    ack_sender,
                }))
                .await
                .unwrap();

            debug!(?uuid, %username, "Player connected");
        }
    }
}

async fn handle_logout_request(
    socket_ref: SocketRef,
    data: GameRequest,
    ack: AckSender,
    sender: mpsc::Sender<Event>,
) {
    if let GameRequest::Logout {
        game_uuid,
        player_uuid,
    } = data
    {
        if socket_ref.extensions.get::<Player>().is_some() {
            socket_ref.extensions.remove::<Player>().unwrap();

            sender
                .send(Event::Game(GameEvent::Logout {
                    socket_ref,
                    game_uuid,
                    player_uuid,
                    ack,
                }))
                .await
                .unwrap();
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
        // Player shouldn't register again if already registered
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

async fn handle_player_list_request(
    data: GameRequest,
    ack_sender: AckSender,
    game_uuid: Uuid,
    sender: mpsc::Sender<Event>,
) {
    if let GameRequest::PlayerList {} = data {
        sender
            .send(Event::Game(GameEvent::PlayerList {
                game_uuid,
                ack_sender,
            }))
            .await
            .unwrap()
    }
}

async fn handle_start_game_request(
    data: GameRequest,
    game_uuid: Uuid,
    sender: mpsc::Sender<Event>,
) {
    if let GameRequest::Start {} = data {
        sender
            .send(Event::Game(GameEvent::StartGame { game_uuid }))
            .await
            .unwrap()
    }
}

pub fn on_connect(socket: SocketRef, sender: mpsc::Sender<Event>, game_uuid: Uuid) {
    let sender_clone_1 = sender.clone();
    let sender_clone_2 = sender.clone();
    let sender_clone_3 = sender.clone();
    let sender_clone_4 = sender.clone();
    let sender_clone_5 = sender.clone();

    socket.on(
        "register_request",
        |socket: SocketRef, Data::<GameRequest>(data), ack_sender: AckSender| async move {
            handle_registration_request(socket, data, ack_sender, sender_clone_1).await;
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

    socket.on(
        "player-list",
        move |socket_ref: SocketRef, Data::<GameRequest>(data), ack_sender: AckSender| async move {
            handle_player_list_request(data, ack_sender, game_uuid, sender_clone_4).await;
        },
    );

    socket.on(
        "start",
        move |socket: SocketRef, Data::<GameRequest>(data), ack: AckSender| async move {
            handle_start_game_request(data, game_uuid, sender_clone_5).await;
        },
    )
}

pub fn handle_events(event: Event, socket_io: &SocketIo, manager: &mut Manager) {
    if let Game(event) = event {
        match event {
            // A player found a game and decided to play
            GameEvent::Registration {
                socket_ref,
                game_uuid,
                player,
                ack_sender,
            } => {
                let player_response = match manager.register_player_to_game(&game_uuid, player) {
                    Ok(player) => Response::from_data(player),
                    Err(error) => Response::from_error(error),
                };
                ack_sender.send(&player_response).unwrap();

                let players_response =
                    Response::from_data(manager.get_players_for_game(&game_uuid));
                socket_ref
                    .broadcast()
                    .emit("players-list", &players_response)
                    .ok();
                socket_ref.emit("players-list", &players_response).ok();
            }

            // A player hit the `Log out` button
            GameEvent::Logout {
                socket_ref,
                game_uuid,
                player_uuid,
                ack,
            } => {
                let player_response =
                    match manager.remove_player_from_game(&game_uuid, &player_uuid) {
                        Err(error) => crate::response::Response::from_error(error),
                        Ok(_) => {
                            let players_response =
                                Response::from_data(manager.get_players_for_game(&game_uuid));
                            socket_ref
                                .broadcast()
                                .emit("players-list", &players_response)
                                .ok();
                            socket_ref.emit("players-list", &players_response).ok();

                            Response::from_data("Player successfully removed")
                        }
                    };
                ack.send(&player_response).unwrap();
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

            // A client would like to see who's in the game lobby
            GameEvent::PlayerList {
                game_uuid,
                ack_sender,
            } => {
                let response = Response::from_data(manager.get_players_for_game(&game_uuid));
                ack_sender.send(&response).unwrap();
            }

            // Game is started, tiles are given to the players
            GameEvent::StartGame { game_uuid } => {
                if let Ok(racks) = manager.start_game(&game_uuid) {
                    let sockets_iter = socket_io
                        .of(format!("/game/{game_uuid}"))
                        .unwrap()
                        .sockets()
                        .unwrap_or(Vec::new())
                        .into_iter();

                    for socket in sockets_iter {
                        if let Some(player) = socket.extensions.get::<Player>() {
                            if let Some(rack) = racks.get(player.get_id()) {
                                socket.emit("get-tiles", rack).unwrap()
                            }
                        }
                    }
                }
            }
        }
    }
}

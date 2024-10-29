mod player;
mod game;
mod events;
mod manager;

use std::fmt::Formatter;
use socketioxide::extract::{AckSender, Data, SocketRef};
use socketioxide::SocketIo;
use tower_http::cors::CorsLayer;
use axum::Router;
use axum::routing::get;
use tracing::{debug, Level};
use tracing_subscriber::FmtSubscriber;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::player::Player;
use crate::events::Event;
use crate::manager::Manager;

#[derive(Copy, Clone, PartialEq)]
struct Tile(char, usize);

#[derive(Debug)]
enum Error {
    NotEnoughPlayers,
    TooManyPlayer,
    DuplicatePlayerId,
    PlayerNotRegistered,
    NoMoreTiles,
    PlayerHas7Tiles,
    GameNotFound
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotEnoughPlayers => write!(f, "Not enough players"),
            Error::TooManyPlayer => write!(f, "Too many players"),
            Error::DuplicatePlayerId => write!(f, "Duplicate player UUID"),
            Error::PlayerNotRegistered => write!(f, "Player is not registered in this game"),
            Error::NoMoreTiles => write!(f, "No more tiles in the bag"),
            Error::GameNotFound => write!(f, "Game not found with this UUID"),
            Error::PlayerHas7Tiles => write!(f, "Player already has 7 tiles"),
        }
    }
}

struct Play {
    tile: Tile,
    x: usize,
    y: usize,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase", untagged)]
enum Messages {
    RegisterRequest {
        game_uuid: Uuid,
        username: String
    },
    LogoutRequest {
        game_uuid: Uuid,
        player_uuid: Uuid,
    }
}

async fn handle_registration_request(
    socket: SocketRef,
    data: Messages,
    ack: AckSender,
    sender: mpsc::Sender<Event>,
) {
    if let Messages::RegisterRequest { game_uuid, username } = data {
        if socket.extensions.get::<Player>().is_none() {
            let uuid = Uuid::new_v4();
            let player = Player::new(&uuid, &username);

            // Inform the manager there's a new player
            sender.send(Event::Registration {
                game_uuid,
                player: player.clone(),
                ack
            }).await.unwrap();

            // Associate the player to the socket for easy access
            socket.extensions.insert::<Player>(player).unwrap();

            debug!(?uuid, %username, "Player connected");
        }
    }
}

async fn handle_logout_request(
    socket: SocketRef,
    data: Messages,
    ack: AckSender,
    sender: mpsc::Sender<Event>,
) {
    if let Messages::LogoutRequest { game_uuid, player_uuid } = data {
        if socket.extensions.get::<Player>().is_some() {
            sender.send(Event::Logout {
                game_uuid,
                player_uuid,
                ack
            }).await.unwrap();

            socket.extensions.remove::<Player>().unwrap();
        }
    }
}

fn on_connect(socket: SocketRef, sender: mpsc::Sender<Event>) {
    let sender_clone_1 = sender.clone();
    let sender_clone_2 = sender.clone();

    socket.on("register_request",  |socket: SocketRef, Data::<Messages>(data), ack: AckSender| async move {
        handle_registration_request(socket, data, ack, sender_clone_1).await;
    });

    socket.on("logout", |socket: SocketRef, Data::<Messages>(data), ack: AckSender| async move {
        handle_logout_request(socket, data, ack, sender_clone_2).await;
    });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let mut manager = Manager::new();

    let (tx, mut rx) = mpsc::channel::<Event>(32);

    let (layer, io) = SocketIo::new_layer();

    io.ns("/", move |socket| on_connect(socket, tx.clone()));

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .layer(layer)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                // A player found a game and decided to play
                Event::Registration { game_uuid, player, ack  } => {
                    let player = manager.register_player_to_game(&game_uuid, player).unwrap();
                    ack.send(player.get_id()).ok();
                },

                // A player hit the `Log out` button
                Event::Logout { game_uuid, player_uuid, ack } => {
                    let message = match manager.remove_player_from_game(&game_uuid, &player_uuid) {
                        Err(error) => error.to_string(),
                        Ok(_) => String::from("Player successfully removed")
                    };

                    ack.send(&message).unwrap();
                }
            }
        }
    });

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
mod player;
mod game;

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use socketioxide::extract::{AckSender, Data, SocketRef};
use socketioxide::SocketIo;
use tower_http::cors::CorsLayer;
use rmpv::Value;
use axum::Router;
use axum::routing::get;
use tracing::{debug, info, Level};
use tracing_subscriber::FmtSubscriber;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::field::debug;
use uuid::Uuid;
use crate::game::Game;
use crate::player::Player;

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
    LoginRequest {
        uuid: Uuid,
    }
}

enum Event {
    Registration {
        game_uuid: Uuid,
        player: Player,
        ack: AckSender
    }
}

struct Manager(HashMap<Uuid, Game>);
impl Manager {
    fn new() -> Self {
        let mut result = Self(HashMap::new());

        // For testing purpose
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        result.0.insert(uuid, Game::new());

        result
    }

    fn create_game(&mut self) -> Uuid {
        let uuid = Uuid::new_v4();
        self.0.insert(uuid, Game::new());

        uuid
    }

    fn register_player_to_game(&mut self, game_uuid: &Uuid, player: Player) -> Result<&Player, Error> {
        match self.0.get_mut(game_uuid) {
            Some(game) => Ok(game.register_player(player)?),
            None => Err(Error::GameNotFound)
        }
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
            socket.extensions.insert(player);

            debug!(?uuid, %username, "Player connected");
        }
    }
}


fn on_connect(socket: SocketRef, sender: mpsc::Sender<Event>) {
    socket.on("register_request",  |socket: SocketRef, Data::<Messages>(data), ack: AckSender| async move {
        handle_registration_request(socket, data, ack, sender).await;
    });

    socket.on("logout", |socket: SocketRef| {
        match socket.extensions.get::<Player>() {
            None => debug!("Player is already disconnected"),
            Some(_) => {
                socket.extensions.remove::<Player>();
                debug!("Player disconnected");
            }
        };
    });

    socket.on("message", |socket: SocketRef, Data::<Value>(data)| {
        info!(?data, "Received event:");
        socket.emit("message-back", &data).ok();
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
                }
            }
        }
    });

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
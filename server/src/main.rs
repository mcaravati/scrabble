mod player;
mod game;

use std::collections::HashMap;
use socketioxide::extract::{AckSender, Data, SocketRef};
use socketioxide::SocketIo;
use tower_http::cors::CorsLayer;
use rmpv::Value;
use axum::Router;
use axum::routing::get;
use tracing::{debug, info, Level};
use tracing_subscriber::FmtSubscriber;
use serde::{Deserialize, Serialize};
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
    PlayerHas7Tiles
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
        username: String
    },
    LoginRequest {
        uuid: Uuid,
    }
}

struct Manager(HashMap<Uuid, Game>);
impl Manager {
    fn new() -> Self {
        Self(HashMap::new())
    }

    fn create_game(&mut self) {
        let uuid = Uuid::new_v4();
        self.0.insert(uuid, Game::new());
    }
}

fn on_connect(socket: SocketRef, Data(data): Data<Value>) {
    socket.on("register_request", move |socket: SocketRef, Data::<Messages>(data), ack: AckSender| {
        match &data {
            Messages::RegisterRequest { username } => {
                if socket.extensions.get::<Player>().is_some() {
                    return;
                }

                let uuid = Uuid::new_v4();
                socket.extensions.insert(Player::new(&uuid, &username));

                debug!(?uuid, %username, "Player connected");
                ack.send(&uuid).ok();
            },
            _ => {}
        }
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

    socket.on("message-with-ack", |Data::<Value>(data), ack: AckSender| {
        info!(?data, "Received event");
        ack.send(&data).ok();
    });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let (layer, io) = SocketIo::new_layer();

    io.ns("/", on_connect);

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .layer(layer)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
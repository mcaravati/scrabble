mod events;
mod game;
mod lobby;
mod manager;
mod player;
mod response;
mod scrabble;

use crate::events::Event;
use crate::manager::Manager;
use axum::routing::get;
use axum::Router;
use serde::{Deserialize, Serialize, Serializer};
use socketioxide::extract::SocketRef;
use socketioxide::SocketIo;
use std::fmt::Formatter;
use std::str::FromStr;
use tokio::sync::mpsc;
use tower_http::cors::CorsLayer;
use tracing::{debug, Level};
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, Debug)]
struct Tile(char, usize);

#[derive(Debug, PartialEq)]
enum Error {
    NotEnoughPlayers,
    TooManyPlayer,
    DuplicatePlayerId,
    PlayerNotRegistered,
    NoMoreTiles,
    PlayerHas7Tiles,
    GameNotFound,
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
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

fn on_game_namespace_connect(socket: SocketRef, sender: mpsc::Sender<Event>) {
    debug!("Connecting game namespace");
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

    io.ns("/", {
        let sender = tx.clone();
        move |socket| crate::lobby::on_connect(socket, sender)
    });
    io.dyn_ns("/game/{*game_uuid}", {
        let sender = tx.clone();
        move |socket_ref: SocketRef| {
            let ns = socket_ref.ns();
            let game_uuid = Uuid::from_str(ns.split("/").last().unwrap()).unwrap();

            return crate::game::on_connect(socket_ref, sender, game_uuid);
        }
    })
    .unwrap();

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .layer(layer)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                Event::Game(_) => crate::game::handle_events(event, &io, &mut manager),
                Event::Lobby(_) => crate::lobby::handle_events(event, &mut manager),
            }
        }
    });

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

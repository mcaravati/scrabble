use crate::game::GameEvent;
use crate::lobby::LobbyEvent;

pub enum Event {
    Lobby(LobbyEvent),
    Game(GameEvent),
}

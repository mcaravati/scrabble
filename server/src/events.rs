use socketioxide::extract::{AckSender, SocketRef};
use uuid::Uuid;
use crate::player::Player;

pub enum Event {
    Registration {
        game_uuid: Uuid,
        player: Player,
        ack: AckSender
    },
    Logout {
        game_uuid: Uuid,
        player_uuid: Uuid,
        ack: AckSender
    },
    ListGames {
        ack_sender: AckSender
    },
    WhoAmI {
        socket_ref: SocketRef,
        player_uuid: Uuid,
        ack_sender: AckSender
    }
}

use socketioxide::extract::AckSender;
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
    }
}

use std::collections::HashMap;
use uuid::Uuid;
use crate::Error;
use crate::game::Game;
use crate::player::Player;

pub struct Manager(HashMap<Uuid, Game>);
impl Manager {
    pub fn new() -> Self {
        let mut result = Self(HashMap::new());

        // For testing purpose
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        result.0.insert(uuid, Game::new());

        result
    }

    pub fn create_game(&mut self) -> Uuid {
        let uuid = Uuid::new_v4();
        self.0.insert(uuid, Game::new());

        uuid
    }

    pub fn register_player_to_game(&mut self, game_uuid: &Uuid, player: Player) -> Result<&Player, Error> {
        match self.0.get_mut(game_uuid) {
            Some(game) => Ok(game.register_player(player)?),
            None => Err(Error::GameNotFound)
        }
    }

    pub fn remove_player_from_game(&mut self, game_uuid: &Uuid, player_uuid: &Uuid) -> Result<(), Error> {
        match self.0.get_mut(game_uuid) {
            Some(game) => Ok(game.remove_player(player_uuid)),
            None => Err(Error::GameNotFound)
        }
    }
}
use crate::player::Player;
use crate::scrabble::Scrabble;
use crate::{Error, Tile};
use std::collections::HashMap;
use uuid::Uuid;

pub struct Manager {
    game_map: HashMap<Uuid, Scrabble>,
    player_to_game: HashMap<Uuid, Uuid>,
}

impl Manager {
    pub fn new() -> Self {
        let mut result = Self {
            game_map: HashMap::new(),
            player_to_game: HashMap::new(),
        };

        // For testing purpose
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        result.game_map.insert(uuid, Scrabble::new());

        result
    }

    pub fn create_game(&mut self) -> Uuid {
        let uuid = Uuid::new_v4();
        self.game_map.insert(uuid, Scrabble::new());

        uuid
    }

    pub fn register_player_to_game(
        &mut self,
        game_uuid: &Uuid,
        player: Player,
    ) -> Result<&Player, Error> {
        match self.game_map.get_mut(game_uuid) {
            Some(game) => {
                let registered_player = game.register_player(player)?;
                self.player_to_game
                    .insert(registered_player.get_id().clone(), game_uuid.clone());

                Ok(registered_player)
            }
            None => Err(Error::GameNotFound),
        }
    }

    pub fn remove_player_from_game(
        &mut self,
        game_uuid: &Uuid,
        player_uuid: &Uuid,
    ) -> Result<(), Error> {
        match self.game_map.get_mut(game_uuid) {
            Some(game) => game.remove_player(player_uuid),
            None => Err(Error::GameNotFound),
        }
    }

    pub fn get_game_list(&self) -> Vec<&Uuid> {
        self.game_map.keys().collect()
    }

    pub fn player_from_uuid(&self, player_uuid: &Uuid) -> Result<&Player, Error> {
        match self.player_to_game.get(player_uuid) {
            Some(game_uuid) => match self.game_map.get(game_uuid) {
                Some(game) => Ok(game.get_player(player_uuid)?),
                None => Err(Error::GameNotFound),
            },
            None => Err(Error::PlayerNotRegistered),
        }
    }

    pub fn get_players_for_game(&self, game_uuid: &Uuid) -> Vec<Player> {
        match self.game_map.get(game_uuid) {
            Some(game) => game.get_players(),
            None => Vec::new(),
        }
    }

    pub fn start_game(&mut self, game_uuid: &Uuid) -> Result<HashMap<Uuid, Vec<Tile>>, Error> {
        match self.game_map.get_mut(game_uuid) {
            Some(game) => game.start(),
            None => Err(Error::GameNotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_is_added_on_creation() {
        let mut manager = Manager::new();

        assert_eq!(manager.game_map.len(), 0);

        let game = manager.create_game();

        assert_eq!(manager.game_map.len(), 1);
    }
}

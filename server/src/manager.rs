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
        // let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        // result.game_map.insert(uuid, Scrabble::new());

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
    use std::ops::Deref;

    fn create_player() -> Player {
        let player_id = Uuid::new_v4();
        let player_name = format!("Player_{player_id}");

        Player::new(&player_id, &player_name)
    }

    #[test]
    fn game_is_added_on_creation() {
        let mut manager = Manager::new();

        assert_eq!(manager.game_map.len(), 0);

        let game = manager.create_game();

        assert_eq!(manager.game_map.len(), 1);
    }

    #[test]
    fn register_player_to_game__good_id() {
        let mut manager = Manager::new();

        let game_uuid = manager.create_game();
        let player = create_player();

        let result = manager.register_player_to_game(&game_uuid, player.clone());
        assert!(result.is_ok());
        assert_eq!(player, result.unwrap().deref().clone());
        assert_eq!(manager.get_players_for_game(&game_uuid), vec![player]);
    }

    #[test]
    fn register_player_to_game__bad_id() {
        let mut manager = Manager::new();

        let game_uuid = manager.create_game();
        let player = create_player();
        let random_id = Uuid::new_v4();

        let result = manager.register_player_to_game(&random_id, player.clone());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::GameNotFound);
        assert_eq!(manager.get_players_for_game(&game_uuid), vec![]);
    }

    #[test]
    fn register_player_to_game__no_games() {
        let mut manager = Manager::new();

        let player = create_player();
        let random_id = Uuid::new_v4();

        let result = manager.register_player_to_game(&random_id, player.clone());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::GameNotFound);
    }

    #[test]
    fn remove_player_from_game__good_id() {
        let mut manager = Manager::new();

        let game_uuid = manager.create_game();
        let player = create_player();

        manager
            .register_player_to_game(&game_uuid, player.clone())
            .unwrap();
        let result = manager.remove_player_from_game(&game_uuid, player.get_id());
        assert!(result.is_ok());
        assert_eq!(manager.get_players_for_game(&game_uuid), vec![]);
    }

    #[test]
    fn remove_player_from_game__bad_ids() {
        {
            let mut manager = Manager::new();

            let game_uuid = manager.create_game();
            let player = create_player();
            let random_id = Uuid::new_v4();

            manager
                .register_player_to_game(&game_uuid, player.clone())
                .unwrap();
            let result = manager.remove_player_from_game(&random_id, player.get_id());
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), Error::GameNotFound);
            assert_eq!(manager.get_players_for_game(&game_uuid), vec![player]);
        }

        {
            let mut manager = Manager::new();

            let game_uuid = manager.create_game();
            let player = create_player();
            let random_id = Uuid::new_v4();

            manager
                .register_player_to_game(&game_uuid, player.clone())
                .unwrap();
            let result = manager.remove_player_from_game(&game_uuid, &random_id);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), Error::PlayerNotRegistered);
            assert_eq!(manager.get_players_for_game(&game_uuid), vec![player]);
        }

        {
            let mut manager = Manager::new();

            let game_uuid = manager.create_game();
            let player = create_player();
            let random_id = Uuid::new_v4();

            manager
                .register_player_to_game(&game_uuid, player.clone())
                .unwrap();
            let result = manager.remove_player_from_game(&random_id, &random_id);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), Error::GameNotFound);
            assert_eq!(manager.get_players_for_game(&game_uuid), vec![player]);
        }
    }

    #[test]
    fn player_from_uuid__good_id() {
        let mut manager = Manager::new();

        let game_uuid = manager.create_game();
        let player = create_player();

        manager
            .register_player_to_game(&game_uuid, player.clone())
            .unwrap();
        let result = manager.player_from_uuid(player.get_id());
        assert!(result.is_ok());
        assert_eq!(player, result.unwrap().deref().clone());
    }

    #[test]
    fn player_from_uuid__bad_id() {
        let mut manager = Manager::new();

        let game_uuid = manager.create_game();
        let player = create_player();
        let random_id = Uuid::new_v4();

        manager
            .register_player_to_game(&game_uuid, player.clone())
            .unwrap();
        let result = manager.player_from_uuid(&random_id);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::PlayerNotRegistered);
    }

    #[test]
    fn get_players_for_game__good_id() {
        let mut manager = Manager::new();

        let game_uuid = manager.create_game();
        let player = create_player();

        manager
            .register_player_to_game(&game_uuid, player.clone())
            .unwrap();
        let result = manager.get_players_for_game(&game_uuid);

        assert_eq!(result.len(), 1);
        assert_eq!(result, vec![player]);
    }

    #[test]
    fn get_players_for_game__bad_id() {
        let mut manager = Manager::new();

        let game_uuid = manager.create_game();
        let player = create_player();
        let random_id = Uuid::new_v4();

        manager
            .register_player_to_game(&game_uuid, player.clone())
            .unwrap();
        let result = manager.get_players_for_game(&random_id);

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn start_game__good_id() {
        let mut manager = Manager::new();

        let game_uuid = manager.create_game();

        let player_1 = create_player();
        let player_2 = create_player();

        manager
            .register_player_to_game(&game_uuid, player_1.clone())
            .unwrap();
        manager
            .register_player_to_game(&game_uuid, player_2.clone())
            .unwrap();
        let result = manager.start_game(&game_uuid);

        assert!(result.is_ok());

        let map = result.unwrap();
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(player_1.get_id()).unwrap().len(), 7);
        assert_eq!(map.get(player_2.get_id()).unwrap().len(), 7);
    }

    #[test]
    fn start_game__not_enough_players() {
        {
            let mut manager = Manager::new();
            let game_uuid = manager.create_game();

            let result = manager.start_game(&game_uuid);

            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), Error::NotEnoughPlayers);
        }

        {
            let mut manager = Manager::new();
            let game_uuid = manager.create_game();

            let player = create_player();

            manager
                .register_player_to_game(&game_uuid, player.clone())
                .unwrap();
            let result = manager.start_game(&game_uuid);

            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), Error::NotEnoughPlayers);
        }
    }

    #[test]
    fn start_game__bad_id() {
        let mut manager = Manager::new();

        let game_uuid = manager.create_game();

        let player_1 = create_player();
        let player_2 = create_player();

        manager
            .register_player_to_game(&game_uuid, player_1.clone())
            .unwrap();
        manager
            .register_player_to_game(&game_uuid, player_2.clone())
            .unwrap();

        let random_uuid = Uuid::new_v4();
        let result = manager.start_game(&random_uuid);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::GameNotFound);
    }
}

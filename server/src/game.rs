use std::collections::HashMap;
use rand::prelude::SliceRandom;
use uuid::Uuid;
use crate::{Error, Tile};
use crate::player::Player;

const TILE_BAG: [(Tile, usize); 26] = [
    (Tile('A', 1), 9), (Tile('B', 3), 2), (Tile('C', 3), 2), (Tile('D', 2), 4),
    (Tile('E', 1), 12), (Tile('F', 4), 2), (Tile('G', 2), 3), (Tile('H', 4), 2),
    (Tile('I', 1), 9), (Tile('J', 8), 1), (Tile('K', 5), 1), (Tile('L', 1), 4),
    (Tile('M', 3), 2), (Tile('N', 1), 6), (Tile('O', 1), 8), (Tile('P', 3), 2),
    (Tile('Q', 10), 1), (Tile('R', 1), 6), (Tile('S', 1), 4), (Tile('T', 1), 6),
    (Tile('U', 1), 4), (Tile('V', 4), 2), (Tile('W', 4), 2), (Tile('X', 8), 1),
    (Tile('Y', 4), 2), (Tile('Z', 10), 1),
];

const BOARD_SIZE: usize = 15;

pub struct Game {
    board: [[char; BOARD_SIZE]; BOARD_SIZE],
    tile_bag: Vec<Tile>,
    racks: HashMap<Uuid, Vec<Tile>>,
    players: Vec<Player>,
    current_player_index: usize,
}

impl Game {
    pub fn new() -> Self {
        let mut game = Game {
            board: [[' ' as char; BOARD_SIZE]; BOARD_SIZE],
            tile_bag: Vec::new(),
            racks: HashMap::new(),
            players: Vec::new(),
            current_player_index: 0,
        };

        game.init_tile_bag();
        game
    }

    fn init_tile_bag(&mut self) {
        for &(tile, amount) in &TILE_BAG {
            for _ in 0..amount {
                self.tile_bag.push(tile);
            }
        }

        let mut rng = rand::thread_rng();
        self.tile_bag.shuffle(&mut rng);
    }

    pub fn register_player(&mut self, player: Player) -> Result<&Player, Error> {
        // No more than 4 players
        if self.players.len() >= 4 {
            return Err(Error::TooManyPlayer);
        } else if self.players.iter().any(|x| x.get_id() == player.get_id()) {
            return Err(Error::DuplicatePlayerId)
        }

        self.racks.insert(*player.get_id(), Vec::new());
        self.players.push(player);

        Ok(self.players.last().unwrap())
    }

    pub fn remove_player(&mut self, player_uuid: &Uuid) {
        self.players.retain(|x| x.get_id() != player_uuid);
        self.racks.remove(player_uuid);
    }

    fn is_player_registered(&self, player: &Player) -> bool {
        self.players.contains(player)
    }

    fn are_there_tiles_remaining(&self) -> bool {
        self.tile_bag.is_empty()
    }

    fn get_player_ids(&self) -> Vec<Uuid> {
        self.players.iter().map(|x| x.get_id().clone()).collect()
    }

    fn give_tile(&mut self, player_id: &Uuid) -> Result<(), Error> {
        let tile = self.tile_bag.pop().ok_or(Error::NoMoreTiles)?;

        match self.racks.get_mut(player_id) {
            Some(rack) => {
                if rack.len() == 7 {
                    return Err(Error::PlayerHas7Tiles);
                }

                rack.push(tile);
                Ok(())
            },
            None => {
                self.tile_bag.push(tile);
                Err(Error::PlayerNotRegistered)
            }
        }
    }

    pub fn get_player(&self, player_id: &Uuid) -> Result<&Player, Error> {
        self.players.iter().find(|x| x.get_id() == player_id).ok_or(Error::PlayerNotRegistered)
    }

    fn get_player_tiles(&self, player_uuid: &Uuid) -> Result<&Vec<Tile>, Error> {
        match self.racks.get(player_uuid) {
            Some(rack) => Ok(rack),
            None => Err(Error::PlayerNotRegistered)
        }
    }

    fn start(&mut self) -> Result<(), Error> {
        if self.players.len() < 2 {
            return Err(Error::NotEnoughPlayers);
        }

        let player_ids = self.get_player_ids();

        for player_id in player_ids {
            for _ in 0..7 {
                self.give_tile(&player_id)?
            }
        }

        Ok(())
    }

    pub fn display_board(&self) {
        println!("Initial board:");
        for row in self.board {
            print!("|");
            for cell in row {
                print!("{} ", cell);
            }
            println!("|");
        }
    }

    pub fn next_turn(&mut self) -> usize {
        self.current_player_index = (self.current_player_index + 1) % self.players.len();

        self.current_player_index
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use crate::Player;
    use super::Game;

    // === Game.new()

    #[test]
    fn test_new_game_tile_bag_initialized() {
        let game = Game::new();
        assert_eq!(game.tile_bag.len(), 98); // Official rules
    }

    // ===

    #[test]
    fn game_cannot_have_more_than_4_players() {
        let mut game = Game::new();

        for _ in 0..4 {
            let uuid = Uuid::new_v4();
            assert!(game.register_player(Player::new(&uuid, "Player")).is_ok());
        }

        let uuid = Uuid::new_v4();
        assert!(game.register_player(Player::new(&uuid, "Player")).is_err());
    }

    #[test]
    fn game_cannot_have_multiple_players_with_same_id() {
        let mut game = Game::new();

        let uuid = Uuid::new_v4();
        assert!(game.register_player(Player::new(&uuid, "Player")).is_ok());

        let uuid = Uuid::new_v4();
        assert!(game.register_player(Player::new(&uuid, "Player")).is_ok());
        assert!(game.register_player(Player::new(&uuid, "Player")).is_err());
    }

    #[test]
    fn game_cannot_start_without_at_least_2_players() {
        {
            let mut game = Game::new();

            // Zero players
            assert!(game.start().is_err());
        }

        {
            let mut game = Game::new();

            // One player
            let uuid = Uuid::new_v4();
            game.register_player(Player::new(&uuid, "Player")).unwrap();
            assert!(game.start().is_err());
        }

        // >= 2 players
        for n_players in 2..4 {
            let mut game = Game::new();

            let uuid = Uuid::new_v4();
            game.register_player(Player::new(&uuid, "Player")).unwrap();

            for _ in 1..n_players {
                let uuid = Uuid::new_v4();
                game.register_player(Player::new(&uuid, "Player")).unwrap();
            }

            assert!(game.start().is_ok());
        }
    }

    #[test]
    fn game_can_give_a_registered_player() {
        let mut game = Game::new();

        let uuid_0 = Uuid::new_v4();
        let uuid_1 = Uuid::new_v4();

        assert!(game.register_player(Player::new(&uuid_0, "Player0")).is_ok());
        assert!(game.register_player(Player::new(&uuid_1, "Player1")).is_ok());

        assert!(game.get_player(&uuid_0).is_ok());
        assert_eq!(*game.get_player(&uuid_0).unwrap().get_id(), uuid_0);
        assert_eq!(game.get_player(&uuid_0).unwrap().get_name(), "Player0");

        assert!(game.get_player(&uuid_1).is_ok());
        assert_eq!(*game.get_player(&uuid_1).unwrap().get_id(), uuid_1);
        assert_eq!(game.get_player(&uuid_1).unwrap().get_name(), "Player1");
    }

    #[test]
    fn game_cannot_give_player_not_registered() {
        let mut game = Game::new();

        let uuid = Uuid::new_v4();
        assert!(game.register_player(Player::new(&uuid, "Player0")).is_ok());

        let uuid = Uuid::new_v4();
        assert!(game.register_player(Player::new(&uuid, "Player1")).is_ok());

        let uuid = Uuid::new_v4();
        assert!(game.get_player(&uuid).is_err()); // Unknown player ID
    }

    #[test]
    fn game_must_give_7_tiles_on_start() {
        let mut game = Game::new();

        let uuid_0 = Uuid::new_v4();
        let uuid_1 = Uuid::new_v4();

        game.register_player(Player::new(&uuid_0, "Player0")).unwrap();
        game.register_player(Player::new(&uuid_1, "Player1")).unwrap();

        game.start().unwrap();

        assert_eq!(game.get_player_tiles(&uuid_0).unwrap().len(), 7);
        assert_eq!(game.get_player_tiles(&uuid_1).unwrap().len(), 7);
    }

    #[test]
    fn next_turn_works() {
        let mut game = Game::new();

        let uuid_0 = Uuid::new_v4();
        let uuid_1 = Uuid::new_v4();

        game.register_player(Player::new(&uuid_0, "Player0")).unwrap();
        game.register_player(Player::new(&uuid_1, "Player1")).unwrap();

        assert_eq!(game.current_player_index, 0);
        assert_eq!(game.next_turn(), 1);
        assert_eq!(game.next_turn(), 0);
    }
}
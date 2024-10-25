use rand::prelude::SliceRandom;
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
    players: Vec<Player>,
    current_player_index: usize,
}

impl Game {
    pub fn new() -> Self {
        let mut game = Game {
            board: [[' ' as char; BOARD_SIZE]; BOARD_SIZE],
            tile_bag: Vec::new(),
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

    fn register_player(&mut self, player: Player) -> Result<(), Error> {
        // No more than 4 players
        if self.players.len() >= 4 {
            return Err(Error::TooManyPlayer);
        } else if self.players.iter().any(|x| x.get_id() == player.get_id()) {
            return Err(Error::DuplicatePlayerId)
        }

        self.players.push(player);
        Ok(())
    }

    fn is_player_registered(&self, player: &Player) -> bool {
        self.players.contains(player)
    }

    fn are_there_tiles_remaining(&self) -> bool {
        self.tile_bag.is_empty()
    }

    fn get_player_ids(&self) -> Vec<u64> {
        self.players.iter().map(|x| x.get_id()).collect()
    }

    fn give_tile(&mut self, player_id: u64) -> Result<(), Error> {
        let tile = self.tile_bag.pop().ok_or(Error::NoMoreTiles)?;

        if let Ok(player) = self.get_player(player_id) {
            player.add_tile(tile)?;
        } else {
            self.tile_bag.push(tile);
            return Err(Error::PlayerNotRegistered);
        }

        Ok(())
    }

    fn get_player(&mut self, player_id: u64) -> Result<&mut Player, Error> {
        self.players.iter_mut().find(|x| x.get_id() == player_id).ok_or(Error::PlayerNotRegistered)
    }

    fn start(&mut self) -> Result<(), Error> {
        if self.players.len() < 2 {
            return Err(Error::NotEnoughPlayers);
        }

        let player_ids = self.get_player_ids();

        for player_id in player_ids {
            for _ in 0..7 {
                self.give_tile(player_id)?
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
    use crate::{Game, Player};

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

        for id in 0..4 {
            assert!(game.register_player(Player::new(id, "Player")).is_ok());
        }

        assert!(game.register_player(Player::new(4, "Player")).is_err());
    }

    #[test]
    fn game_cannot_have_multiple_players_with_same_id() {
        let mut game = Game::new();

        assert!(game.register_player(Player::new(0, "Player")).is_ok());
        assert!(game.register_player(Player::new(1, "Player")).is_ok());

        assert!(game.register_player(Player::new(1, "Player")).is_err());
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
            game.register_player(Player::new(0, "Player")).unwrap();
            assert!(game.start().is_err());
        }

        // >= 2 players
        for n_players in 2..4 {
            let mut game = Game::new();

            game.register_player(Player::new(0, "Player")).unwrap();
            for id in 1..n_players {
                game.register_player(Player::new(id, "Player")).unwrap();
            }

            assert!(game.start().is_ok());
        }
    }

    #[test]
    fn game_can_give_a_registered_player() {
        let mut game = Game::new();

        assert!(game.register_player(Player::new(0, "Player0")).is_ok());
        assert!(game.register_player(Player::new(1, "Player1")).is_ok());

        assert!(game.get_player(0).is_ok());
        assert_eq!(game.get_player(0).unwrap().get_id(), 0);
        assert_eq!(game.get_player(0).unwrap().get_name(), "Player0");

        assert!(game.get_player(1).is_ok());
        assert_eq!(game.get_player(1).unwrap().get_id(), 1);
        assert_eq!(game.get_player(1).unwrap().get_name(), "Player1");
    }

    #[test]
    fn game_cannot_give_player_not_registered() {
        let mut game = Game::new();

        assert!(game.register_player(Player::new(0, "Player0")).is_ok());
        assert!(game.register_player(Player::new(1, "Player1")).is_ok());

        assert!(game.get_player(999).is_err()); // Unknown player ID
    }

    #[test]
    fn game_must_give_7_tiles_on_start() {
        let mut game = Game::new();

        game.register_player(Player::new(0, "Player1")).unwrap();
        game.register_player(Player::new(1, "Player2")).unwrap();

        game.start().unwrap();

        assert_eq!(game.get_player(0).unwrap().get_number_of_tiles(), 7);
        assert_eq!(game.get_player(1).unwrap().get_number_of_tiles(), 7);
    }

    #[test]
    fn next_turn_works() {
        let mut game = Game::new();

        game.register_player(Player::new(0, "Player1")).unwrap();
        game.register_player(Player::new(1, "Player2")).unwrap();

        assert_eq!(game.current_player_index, 0);
        assert_eq!(game.next_turn(), 1);
        assert_eq!(game.next_turn(), 0);
    }
}
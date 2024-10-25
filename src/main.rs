mod player;
mod game;

use rand::seq::SliceRandom;
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

fn main() {
    let game = Game::new();

    game.display_board();
}
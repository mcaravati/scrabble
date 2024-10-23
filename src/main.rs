use rand::seq::SliceRandom;

const TILE_BAG: [(char, usize, usize); 26] = [
    ('A', 9, 1), ('B', 2, 3), ('C', 2, 3), ('D', 4, 2), ('E', 12, 1),
    ('F', 2, 4), ('G', 3, 2), ('H', 2, 4), ('I', 9, 1), ('J', 1, 8),
    ('K', 1, 5), ('L', 4, 1), ('M', 2, 3), ('N', 6, 1), ('O', 8, 1),
    ('P', 2, 3), ('Q', 1, 10), ('R', 6, 1), ('S', 4, 1), ('T', 6, 1),
    ('U', 4, 1), ('V', 2, 4), ('W', 2, 4), ('X', 1, 8), ('Y', 2, 4),
    ('Z', 1, 10)
];

const BOARD_SIZE: usize = 15;

struct Game {
    board: [[char; BOARD_SIZE]; BOARD_SIZE],
    tile_bag: Vec<(char, usize)>
}

impl Game {
    fn new() -> Self {
        let mut game = Game {
            board: [[' ' as char; BOARD_SIZE]; BOARD_SIZE],
            tile_bag: Vec::new()
        };

        game.init_tile_bag();
        game
    }

    fn init_tile_bag(&mut self) {
        for &(tile_type, count, score) in &TILE_BAG {
            for _ in 0..count {
                self.tile_bag.push((tile_type, score));
            }
        }

        let mut rng = rand::thread_rng();
        self.tile_bag.shuffle(&mut rng);
    }
}

fn main() {
    let game = Game::new();

    println!("Initial board:");
    for row in &game.board {
        for cell in row {
            print!("{} ", cell);
        }
        println!();
    }

    println!("Initial tile bag:");
    for (tile, score) in &game.tile_bag {
        println!("{}: {}", tile, score);
    }
}
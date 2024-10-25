use crate::Tile;
use crate::Error;

pub struct Player {
    id: u64,
    name: String,
    tiles: Vec<Tile>
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.name == other.name
    }
}

impl Player {
    pub fn new(id: u64, name: &str, ) -> Player {
        Player {
            id,
            name: name.to_string(),
            tiles: Vec::new()
        }
    }

    fn has_tile(&self, tile: &Tile) -> bool {
        self.tiles.contains(tile)
    }

    pub fn add_tile(&mut self, tile: Tile) -> Result<(), Error> {
        if self.tiles.len() == 7 {
            return Err(Error::PlayerHas7Tiles);
        }

        self.tiles.push(tile);

        Ok(())
    }

    pub fn get_number_of_tiles(&self) -> usize {
        self.tiles.len()
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }
}
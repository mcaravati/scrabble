use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Player {
    id: Uuid,
    name: String,
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.name == other.name
    }
}

impl Player {
    pub fn new(id: &Uuid, name: &str) -> Player {
        Player {
            id: id.clone(),
            name: name.to_string(),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_id(&self) -> &Uuid {
        &self.id
    }
}

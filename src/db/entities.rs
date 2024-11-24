use rocket::serde::{Deserialize, Serialize};
use surrealdb::sql::{Id, Thing};

#[derive(Serialize, Deserialize)]
pub struct Game {
    pub(crate) id: Thing,
    pub(crate) state: GameState,
}

impl Default for Game {
    fn default() -> Self {
        Game {
            id: Thing::from(("games", Id::rand())),
            state: GameState::Pending,
        }
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Copy, Clone)]
pub enum GameState {
    Pending,
    Ongoing,
    Finished,
    Cancelled,
}

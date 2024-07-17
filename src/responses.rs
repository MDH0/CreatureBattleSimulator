use crate::GameState;
use rocket::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PostGameResponse {
    pub(crate) game_id: String,
    pub(crate) state: GameState,
}

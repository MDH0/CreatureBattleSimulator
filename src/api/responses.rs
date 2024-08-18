use crate::db::entities::GameState;
use rocket::serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct CreateGame {
    pub(crate) trace_id: Uuid,
    pub(crate) game_id: String,
    pub(crate) state: GameState,
}

#[derive(Serialize, Deserialize)]
pub struct JoinGame {
    pub(crate) trace_id: Uuid,
    pub(crate) message: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetGameStatus {
    pub(crate) trace_id: Uuid,
    pub(crate) game_status: GameState,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorMessage {
    pub(crate) trace_id: Uuid,
    pub(crate) error_message: String,
    pub(crate) error_code: Option<u8>,
}

pub mod types {
    use crate::api::responses::{CreateGame, ErrorMessage, GetGameStatus, JoinGame};
    use rocket::response::status;
    use rocket::serde::json::Json;

    //Is there a better way tha using status::Custom??
    pub type ErrorResponse = status::Custom<Json<ErrorMessage>>;
    pub type CreateGameResponse = status::Custom<Json<CreateGame>>;
    pub type JoinGameResponse = status::Custom<Json<JoinGame>>;
    pub type GetGameStatusResponse = status::Custom<Json<GetGameStatus>>;
}

use crate::{responses::PostGameResponse, db::DbConnection, db::Game, GameState};
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;

#[post("/games")]
pub async fn create_game(
    db: &State<DbConnection>,
) -> Result<Json<PostGameResponse>, status::Custom<String>> {
    let game = Game::default();
    let db_result: Result<Vec<Game>, surrealdb::Error> =
        db.conn.create("games").content(game).await;
    match db_result {
        Ok(result) => {
            if result.len() > 1 {
                return Err(status::Custom(
                    Status::InternalServerError,
                    String::from("Something went wrong. Error code: 1"),
                ));
            }
            let game_id = result.get(0).unwrap().id.id.to_string();
            Ok(Json(PostGameResponse {
                game_id,
                state: GameState::Pending,
            }))
        }
        Err(err) => Err(status::Custom(Status::InternalServerError, err.to_string())),
    }
}

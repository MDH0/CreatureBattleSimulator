use crate::{db::DbConnection, db::Game, responses::PostGameResponse, GameState};
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

#[cfg(test)]
mod test {
    use crate::*;
    use rocket::http::Status;
    use rocket::local::asynchronous::Client;

    #[rocket::async_test]
    async fn testing_test() {
        let client = Client::tracked(build_the_rocket().await).await.unwrap();
        let response = client.post(uri!(super::create_game)).dispatch().await;
        assert_eq!(response.status(), Status::Ok);
        let response_message = response
            .into_json::<responses::PostGameResponse>()
            .await
            .expect("Invalid response from server.");
        assert_eq!(response_message.state, GameState::Pending);
    }
}

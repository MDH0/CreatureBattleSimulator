use crate::api::responses::PostGameResponse;
use crate::db::{
    entities::{Game, GameState},
    DbConnection,
};
use rocket::{http::Status, response::status, serde::json::Json, State};

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

//Needs a proper response message.
#[post("/games/<id>")]
pub async fn join_game(
    id: String,
    db: &State<DbConnection>,
) -> Result<String, status::Custom<String>> {
    let game_query: Result<Option<Game>, surrealdb::Error> = db.conn.select(("games", &id)).await;
    //WTF??
    return match game_query {
        Ok(game) => match game {
            None => Err(status::Custom(
                Status::NotFound,
                String::from("Couldn't find the game you're looking for."),
            )),
            Some(mut game) => {
                if game.state != GameState::Pending {
                    return Err(status::Custom(
                        Status::Conflict,
                        String::from("The game is already active."),
                    ));
                }
                game.state = GameState::Ongoing;
                let update_result: Result<Option<Game>, surrealdb::Error> =
                    db.conn.update(("games", &id)).content(game).await;
                if let Err(err) = update_result {
                    return Err(status::Custom(Status::InternalServerError, err.to_string()));
                }
                Ok(String::from("Joined the game."))
            }
        },
        Err(err) => Err(status::Custom(Status::InternalServerError, err.to_string())),
    };
}

#[cfg(test)]
mod test {
    use crate::{api::responses, db::entities::GameState, *};
    use rocket::{http::Status, local::asynchronous::Client};
    /*This uses the current database for testing, which should be extracted.
    Testcontainers look like a very promising solution
    https://testcontainers.com/
    Also, wtf is the name? */
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

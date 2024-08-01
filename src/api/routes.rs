use crate::api::responses::{types::*, CreateGame, ErrorMessage, JoinGame};
use crate::db::{
    entities::{Game, GameState},
    DbConnection,
};
use rocket::{http::Status, response::status, serde::json::Json, State};

#[post("/games")]
pub async fn create_game(db: &State<DbConnection>) -> Result<CreateGameResponse, ErrorResponse> {
    let game = Game::default();
    let db_result: Result<Vec<Game>, surrealdb::Error> =
        db.conn.create("games").content(game).await;
    match db_result {
        Ok(result) => {
            if result.len() > 1 {
                return Err(status::Custom(
                    Status::InternalServerError,
                    Json(ErrorMessage {
                        error_message: String::from("Something went wrong."),
                        error_code: Some(1),
                    }),
                ));
            }
            let game_id = result.get(0).unwrap().id.id.to_string(); //This looks cursed
            Ok(status::Custom(
                Status::Created,
                Json(CreateGame {
                    game_id,
                    state: GameState::Pending,
                }),
            ))
        }
        Err(err) => Err(status::Custom(
            Status::InternalServerError,
            Json(ErrorMessage {
                error_message: err.to_string(),
                error_code: None,
            }),
        )),
    }
}

#[post("/games/<id>")]
pub async fn join_game(
    id: String,
    db: &State<DbConnection>,
) -> Result<JoinGameResponse, ErrorResponse> {
    let game_query: Result<Option<Game>, surrealdb::Error> = db.conn.select(("games", &id)).await;
    //WTF??
    return match game_query {
        Ok(game) => match game {
            None => Err(status::Custom(
                Status::NotFound,
                Json(ErrorMessage {
                    error_message: String::from("Couldn't find the game you're looking for."),
                    error_code: None,
                }),
            )),
            Some(mut game) => {
                if game.state != GameState::Pending {
                    return Err(status::Custom(
                        Status::Conflict,
                        Json(ErrorMessage {
                            error_message: String::from("The game is already active."),
                            error_code: None,
                        }),
                    ));
                }
                game.state = GameState::Ongoing;
                let update_result: Result<Option<Game>, surrealdb::Error> =
                    db.conn.update(("games", &id)).content(game).await;
                if let Err(err) = update_result {
                    return Err(status::Custom(
                        Status::InternalServerError,
                        Json(ErrorMessage {
                            error_message: err.to_string(),
                            error_code: None,
                        }),
                    ));
                }
                Ok(status::Custom(
                    Status::Ok,
                    Json(JoinGame {
                        message: String::from("Joined the game."),
                    }),
                ))
            }
        },
        Err(err) => Err(status::Custom(
            Status::InternalServerError,
            Json(ErrorMessage {
                error_message: err.to_string(),
                error_code: None,
            }),
        )),
    };
}

#[cfg(test)]
mod test {
    use crate::db::entities::Game;
    use crate::{api::responses, db::entities::GameState, *};
    use rocket::{http::Status, local::asynchronous::Client};
    /*This uses the current database for testing, which should be extracted.
    Testcontainers look like a very promising solution
    https://testcontainers.com/
    Also, wtf is the name? */
    #[rocket::async_test]
    async fn test_create_game() {
        let client = Client::tracked(build_the_rocket().await).await.unwrap();

        let response = client.post(uri!(super::create_game)).dispatch().await;

        assert_eq!(response.status(), Status::Created);
        let response_message = response
            .into_json::<responses::CreateGame>()
            .await
            .expect("Invalid response from server.");
        assert_eq!(response_message.state, GameState::Pending);
    }

    #[rocket::async_test]
    async fn test_joining_a_game() {
        let client = Client::tracked(build_the_rocket().await).await.unwrap();
        let db = client.rocket().state::<DbConnection>().unwrap();
        let game = Game::default();
        let _: Vec<Game> = db
            .conn
            .create("games")
            .content(&game)
            .await
            .expect("Creating game failed.");

        let response = client
            .post(uri!(super::join_game(&game.id.id.to_string())))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        let _ = response
            .into_json::<responses::JoinGame>()
            .await
            .expect("Invalid response from server.");
    }

    #[rocket::async_test]
    async fn test_game_does_not_exist() {
        let client = Client::tracked(build_the_rocket().await).await.unwrap();

        let response = client
            .post(uri!(super::join_game(String::from("lmao"))))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::NotFound);
    }

    #[rocket::async_test]
    async fn test_game_is_not_pending() {
        let client = Client::tracked(build_the_rocket().await).await.unwrap();
        let db = client.rocket().state::<DbConnection>().unwrap();
        let mut game = Game::default();
        game.state = GameState::Ongoing;
        let _: Vec<Game> = db
            .conn
            .create("games")
            .content(&game)
            .await
            .expect("Creating game failed.");

        let response = client
            .post(uri!(super::join_game(game.id.id.to_string())))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Conflict);
    }
}
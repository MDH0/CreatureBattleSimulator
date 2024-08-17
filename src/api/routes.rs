use crate::api::responses::{types::*, CreateGame, ErrorMessage, GetGameStatus, JoinGame};
use crate::db::{
    entities::{Game, GameState},
    DbConnection,
};
use rocket::{http::Status, response::status, serde::json::Json, State};

#[post("/games")]
pub async fn create_game(db: &State<DbConnection>) -> Result<CreateGameResponse, ErrorResponse> {
    log::info!("Test");
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
    id: &str,
    db: &State<DbConnection>,
) -> Result<JoinGameResponse, ErrorResponse> {
    let game_query: Result<Option<Game>, surrealdb::Error> = db.conn.select(("games", id)).await;
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
                    db.conn.update(("games", id)).content(game).await;
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

#[get("/games/<id>")]
pub async fn get_game_state(
    id: &str,
    db: &State<DbConnection>,
) -> Result<GetGameStatusResponse, ErrorResponse> {
    let game_query: Result<Option<Game>, surrealdb::Error> = db.conn.select(("games", id)).await;
    return match game_query {
        Ok(game) => {
            return match game {
                None => Err(status::Custom(
                    Status::NotFound,
                    Json(ErrorMessage {
                        error_message: String::from("Couldn't find the game you're looking for."),
                        error_code: None,
                    }),
                )),
                Some(game) => Ok(status::Custom(
                    Status::Ok,
                    Json(GetGameStatus {
                        game_status: game.state,
                    }),
                )),
            }
        }
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
    use testcontainers_modules::{
        surrealdb,
        testcontainers::{runners::AsyncRunner, ImageExt},
    };
    /* I really would like to use this helper method, but for some reasons tests are just stuck doing nothing when using it
    async fn create_client() -> Client {
        let db_instance = surrealdb::SurrealDb::default()
            .with_tag("v1.5.3")
            .start()
            .await
            .expect("Something went wrong. Do you have a container runtime installed?");
        let rocket = rocket::build();
        let config = Config {
            db_url: String::from(format!(
                "127.0.0.1:{}",
                db_instance
                    .get_host_port_ipv4(surrealdb::SURREALDB_PORT)
                    .await
                    .unwrap()
            )),
            username: String::from("root"),
            password: String::from("root"),
        };
        let client = Client::tracked(build_the_rocket(rocket, config).await)
            .await
            .unwrap();
        client
    }*/

    const SURREALDB_VERSION: &str = "v1.5.4";

    #[rocket::async_test]
    async fn test_create_game() {
        let db_instance = surrealdb::SurrealDb::default()
            .with_tag(SURREALDB_VERSION)
            .start()
            .await
            .expect("Something went wrong. Do you have a container runtime installed?");
        let rocket = rocket::build();
        let config = Config {
            db_url: String::from(format!(
                "127.0.0.1:{}",
                db_instance
                    .get_host_port_ipv4(surrealdb::SURREALDB_PORT)
                    .await
                    .unwrap()
            )),
            username: String::from("root"),
            password: String::from("root"),
        };
        let client = Client::tracked(build_the_rocket(rocket, config).await)
            .await
            .unwrap();

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
        let db_instance = surrealdb::SurrealDb::default()
            .with_tag(SURREALDB_VERSION)
            .start()
            .await
            .expect("Something went wrong. Do you have a container runtime installed?");
        let rocket = rocket::build();
        let config = Config {
            db_url: String::from(format!(
                "127.0.0.1:{}",
                db_instance
                    .get_host_port_ipv4(surrealdb::SURREALDB_PORT)
                    .await
                    .unwrap()
            )),
            username: String::from("root"),
            password: String::from("root"),
        };
        let client = Client::tracked(build_the_rocket(rocket, config).await)
            .await
            .unwrap();
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
    async fn test_joining_non_existent_game() {
        let db_instance = surrealdb::SurrealDb::default()
            .with_tag(SURREALDB_VERSION)
            .start()
            .await
            .expect("Something went wrong. Do you have a container runtime installed?");
        let rocket = rocket::build();
        let config = Config {
            db_url: String::from(format!(
                "127.0.0.1:{}",
                db_instance
                    .get_host_port_ipv4(surrealdb::SURREALDB_PORT)
                    .await
                    .unwrap()
            )),
            username: String::from("root"),
            password: String::from("root"),
        };
        let client = Client::tracked(build_the_rocket(rocket, config).await)
            .await
            .unwrap();

        let response = client
            .post(uri!(super::join_game(String::from("lmao"))))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::NotFound);
    }

    #[rocket::async_test]
    async fn test_game_is_not_pending() {
        let db_instance = surrealdb::SurrealDb::default()
            .with_tag(SURREALDB_VERSION)
            .start()
            .await
            .expect("Something went wrong. Do you have a container runtime installed?");
        let rocket = rocket::build();
        let config = Config {
            db_url: String::from(format!(
                "127.0.0.1:{}",
                db_instance
                    .get_host_port_ipv4(surrealdb::SURREALDB_PORT)
                    .await
                    .unwrap()
            )),
            username: String::from("root"),
            password: String::from("root"),
        };
        let client = Client::tracked(build_the_rocket(rocket, config).await)
            .await
            .unwrap();
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

    #[rocket::async_test]
    async fn test_get_valid_game_status() {
        let db_instance = surrealdb::SurrealDb::default()
            .with_tag(SURREALDB_VERSION)
            .start()
            .await
            .expect("Something went wrong. Do you have a container runtime installed?");
        let rocket = rocket::build();
        let config = Config {
            db_url: String::from(format!(
                "127.0.0.1:{}",
                db_instance
                    .get_host_port_ipv4(surrealdb::SURREALDB_PORT)
                    .await
                    .unwrap()
            )),
            username: String::from("root"),
            password: String::from("root"),
        };
        let client = Client::tracked(build_the_rocket(rocket, config).await)
            .await
            .unwrap();
        let db = client.rocket().state::<DbConnection>().unwrap();
        let game1 = Game::default();
        let mut game2 = Game::default();
        game2.state = GameState::Ongoing;
        let mut game3 = Game::default();
        game3.state = GameState::Finished;
        let games = vec![game1, game2, game3];

        for game in games {
            let _: Vec<Game> = db
                .conn
                .create("games")
                .content(&game)
                .await
                .expect("Creating game failed.");
            let response = client
                .get(uri!(super::get_game_state(game.id.id.to_string())))
                .dispatch()
                .await;

            assert_eq!(response.status(), Status::Ok);
            let response = response
                .into_json::<responses::GetGameStatus>()
                .await
                .expect("Invalid response from server.");
            assert_eq!(response.game_status, game.state);
        }
    }

    #[rocket::async_test]
    async fn test_getting_status_of_non_existent_game() {
        let db_instance = surrealdb::SurrealDb::default()
            .with_tag(SURREALDB_VERSION)
            .start()
            .await
            .expect("Something went wrong. Do you have a container runtime installed?");
        let rocket = rocket::build();
        let config = Config {
            db_url: String::from(format!(
                "127.0.0.1:{}",
                db_instance
                    .get_host_port_ipv4(surrealdb::SURREALDB_PORT)
                    .await
                    .unwrap()
            )),
            username: String::from("root"),
            password: String::from("root"),
        };
        let client = Client::tracked(build_the_rocket(rocket, config).await)
            .await
            .unwrap();

        let response = client
            .get(uri!(super::get_game_state("Somerandomstring")))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::NotFound);
    }
}

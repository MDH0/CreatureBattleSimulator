use crate::api::responses::{
    types::*, CancelGame, CreateGame, ErrorMessage, GetGameStatus, JoinGame,
};
use crate::db::{
    entities::{Game, GameState},
    DbConnection,
};
use rocket::{http::Status, response::status, serde::json::Json, State};
use uuid::Uuid;

#[post("/games")]
pub async fn create_game(db: &State<DbConnection>) -> Result<CreateGameResponse, ErrorResponse> {
    let trace_id = Uuid::new_v4();
    /*
    Sending logs with a trace_id like this is currently just a workaround.
    https://github.com/estk/log4rs/pull/362 should make it more clean in the future.
     */
    log::info!("{} | Received create game request", trace_id.to_string());
    let game = Game::default();
    let db_result: Result<Vec<Game>, surrealdb::Error> =
        db.conn.create("games").content(game).await;
    match db_result {
        Ok(result) => {
            if result.len() > 1 {
                log::error!(
                    "{} | {}",
                    trace_id.to_string(),
                    "Error 1: Request started more than 1 game"
                );
                return Err(status::Custom(
                    Status::InternalServerError,
                    Json(ErrorMessage {
                        trace_id,
                        error_message: String::from("Something went wrong."),
                        error_code: Some(1),
                    }),
                ));
            }
            let game_id = result.get(0).unwrap().id.id.to_string(); //This looks cursed
            log::info!(
                "{} | Created game with id: {}",
                trace_id.to_string(),
                game_id
            );
            Ok(status::Custom(
                Status::Created,
                Json(CreateGame {
                    trace_id,
                    game_id,
                    state: GameState::Pending,
                }),
            ))
        }
        Err(err) => {
            log::error!("{} | {}", trace_id.to_string(), err.to_string());
            Err(status::Custom(
                Status::InternalServerError,
                Json(ErrorMessage {
                    trace_id,
                    error_message: err.to_string(),
                    error_code: None,
                }),
            ))
        }
    }
}

#[put("/games/<id>")]
pub async fn join_game(
    id: &str,
    db: &State<DbConnection>,
) -> Result<JoinGameResponse, ErrorResponse> {
    let trace_id = Uuid::new_v4();
    log::info!(
        "{} | Received join game request for id: {}",
        trace_id.to_string(),
        id
    );
    let game_query: Result<Option<Game>, surrealdb::Error> = db.conn.select(("games", id)).await;
    //WTF??
    return match game_query {
        Ok(game) => match game {
            None => {
                log::info!("{} | Game with id {} does not exist", trace_id, id);
                Err(status::Custom(
                    Status::NotFound,
                    Json(ErrorMessage {
                        trace_id,
                        error_message: String::from("Couldn't find the game you're looking for."),
                        error_code: None,
                    }),
                ))
            }
            Some(mut game) => {
                if game.state != GameState::Pending {
                    log::info!(
                        "{} | Game with id {} does exist, but is not available to join.",
                        trace_id.to_string(),
                        id
                    );
                    return Err(status::Custom(
                        Status::Conflict,
                        Json(ErrorMessage {
                            trace_id,
                            error_message: String::from("The game is already active or finished."),
                            error_code: None,
                        }),
                    ));
                }
                game.state = GameState::Ongoing;
                let update_result: Result<Option<Game>, surrealdb::Error> =
                    db.conn.update(("games", id)).content(game).await;
                if let Err(err) = update_result {
                    log::error!("{} | {}", trace_id.to_string(), err.to_string());
                    return Err(status::Custom(
                        Status::InternalServerError,
                        Json(ErrorMessage {
                            trace_id,
                            error_message: err.to_string(),
                            error_code: None,
                        }),
                    ));
                }
                log::info!("{} | Joined game with id {}", trace_id.to_string(), id);
                Ok(status::Custom(
                    Status::Ok,
                    Json(JoinGame {
                        trace_id,
                        message: String::from("Joined the game."),
                    }),
                ))
            }
        },
        Err(err) => {
            log::error!("{} | {}", trace_id.to_string(), err.to_string());
            Err(status::Custom(
                Status::InternalServerError,
                Json(ErrorMessage {
                    trace_id,
                    error_message: err.to_string(),
                    error_code: None,
                }),
            ))
        }
    };
}

#[get("/games/<id>")]
pub async fn get_game_state(
    id: &str,
    db: &State<DbConnection>,
) -> Result<GetGameStatusResponse, ErrorResponse> {
    let trace_id = Uuid::new_v4();
    log::info!(
        "{} | Received get game status request for id: {}",
        trace_id.to_string(),
        id
    );
    let game_query: Result<Option<Game>, surrealdb::Error> = db.conn.select(("games", id)).await;
    match game_query {
        Ok(game) => match game {
            None => {
                log::info!("{} | Game with id {} does not exist", trace_id, id);
                Err(status::Custom(
                    Status::NotFound,
                    Json(ErrorMessage {
                        trace_id,
                        error_message: String::from("Couldn't find the game you're looking for."),
                        error_code: None,
                    }),
                ))
            }
            Some(game) => {
                log::info!("{} | Received game status {:?}", trace_id, game.state);
                Ok(status::Custom(
                    Status::Ok,
                    Json(GetGameStatus {
                        trace_id,
                        game_status: game.state,
                    }),
                ))
            }
        },
        Err(err) => {
            log::error!("{} | {}", trace_id.to_string(), err.to_string());
            Err(status::Custom(
                Status::InternalServerError,
                Json(ErrorMessage {
                    trace_id,
                    error_message: err.to_string(),
                    error_code: None,
                }),
            ))
        }
    }
}

#[put("/games/<id>/cancel")]
pub async fn cancel_game(
    id: &str,
    db: &State<DbConnection>,
) -> Result<CancelGameResponse, ErrorResponse> {
    let trace_id = Uuid::new_v4();
    log::info!(
        "{} | Received cancel game request for id: {}",
        trace_id.to_string(),
        id
    );
    let game_query: Result<Option<Game>, surrealdb::Error> = db.conn.select(("games", id)).await;
    match game_query {
        Ok(game) => match game {
            None => {
                log::info!("{} | Game with id {} does not exist", trace_id, id);
                Err(status::Custom(
                    Status::NotFound,
                    Json(ErrorMessage {
                        trace_id,
                        error_message: String::from("Couldn't find the game you're looking for."),
                        error_code: None,
                    }),
                ))
            }
            Some(mut game) => match game.state {
                GameState::Pending | GameState::Ongoing => {
                    game.state = GameState::Cancelled;
                    let update_result: Result<Option<Game>, surrealdb::Error> =
                        db.conn.update(("games", id)).content(game).await;
                    if let Err(err) = update_result {
                        log::error!("{} | {}", trace_id.to_string(), err.to_string());
                        return Err(status::Custom(
                            Status::InternalServerError,
                            Json(ErrorMessage {
                                trace_id,
                                error_message: err.to_string(),
                                error_code: None,
                            }),
                        ));
                    }
                    log::info!("{} | Cancelled the game with id {}", trace_id, id);
                    Ok(status::Custom(Status::Ok, Json(CancelGame { trace_id })))
                }
                _ => {
                    log::info!("{} | Game with id {} can not be cancelled", trace_id, id);
                    Err(status::Custom(
                                Status::Conflict,
                                Json(ErrorMessage {
                                    trace_id,
                                    error_message: String::from("Game can not be cancelled. Either the game is already cancelled or it is already finished."),
                                    error_code: None,
                                }),
                            ))
                }
            },
        },
        Err(err) => {
            log::error!("{} | {}", trace_id.to_string(), err.to_string());
            Err(status::Custom(
                Status::InternalServerError,
                Json(ErrorMessage {
                    trace_id,
                    error_message: err.to_string(),
                    error_code: None,
                }),
            ))
        }
    }
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
            .put(uri!(super::join_game(&game.id.id.to_string())))
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
            .put(uri!(super::join_game(String::from("lmao"))))
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
            .put(uri!(super::join_game(game.id.id.to_string())))
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

    #[rocket::async_test]
    async fn test_cancelling_game() {
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
        let games = vec![game1, game2];

        for game in games {
            let _: Vec<Game> = db
                .conn
                .create("games")
                .content(&game)
                .await
                .expect("Creating game failed.");
            let response = client
                .put(uri!(super::cancel_game(game.id.id.to_string())))
                .dispatch()
                .await;

            assert_eq!(response.status(), Status::Ok);
        }

        let games: Vec<Game> = db.conn.select("games").await.unwrap();
        for game in games {
            assert_eq!(game.state, GameState::Cancelled);
        }
    }

    #[rocket::async_test]
    async fn test_cancelling_non_existent_game() {
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
            .put(uri!(super::cancel_game("ajlksdaf")))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::NotFound);
    }

    #[rocket::async_test]
    async fn test_cancelling_uncancellabel_game() {
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
        let mut game1 = Game::default();
        game1.state = GameState::Cancelled;
        let mut game2 = Game::default();
        game2.state = GameState::Finished;
        let games = vec![game1, game2];

        for game in games {
            let _: Vec<Game> = db
                .conn
                .create("games")
                .content(&game)
                .await
                .expect("Creating game failed.");
            let response = client
                .put(uri!(super::cancel_game(game.id.id.to_string())))
                .dispatch()
                .await;

            assert_eq!(response.status(), Status::Conflict);
        }
    }
}

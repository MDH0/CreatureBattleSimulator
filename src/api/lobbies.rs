use crate::{
    api::responses::{
        types::{
            CancelGameResponse, CreateGameResponse, ErrorResponse, GetGameStatusResponse,
            JoinGameResponse,
        },
        CancelGame, CreateGame, ErrorMessage, GetGameStatus, JoinGame,
    },
    db::{entities::GameState, DbConnection},
};
use rocket::{http::Status, response::status, serde::json::Json, State};
use uuid::Uuid;

#[post("/games")]
pub(crate) async fn create_game(
    db: &State<DbConnection>,
) -> Result<CreateGameResponse, ErrorResponse> {
    let trace_id = Uuid::new_v4();
    /*
    Sending logs with a trace_id like this is currently just a workaround.
    https://github.com/estk/log4rs/pull/362 should make it more clean in the future.
     */
    log::info!("{} | Received create game request", trace_id.to_string());
    match db.create_game().await {
        Ok(game_id) => {
            log::info!(
                "{} | Created game with id: {}",
                trace_id.to_string(),
                game_id
            );
            Ok(status::Custom(
                Status::Created,
                Json(CreateGame { trace_id, game_id }),
            ))
        }
        Err(err) => {
            log::error!("{} | {}", trace_id.to_string(), err.message);
            Err(status::Custom(
                err.status_code,
                Json(ErrorMessage {
                    trace_id,
                    error_message: err.message,
                    error_code: None,
                }),
            ))
        }
    }
}

#[put("/games/<id>")]
pub(crate) async fn join_game(
    id: &str,
    db: &State<DbConnection>,
) -> Result<JoinGameResponse, ErrorResponse> {
    let trace_id = Uuid::new_v4();
    log::info!(
        "{} | Received join game request for id: {}",
        trace_id.to_string(),
        id
    );
    match db.get_game(id).await {
        Ok(mut game) => {
            if game.state != GameState::Pending {
                log::error!(
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
            match db.update_game(game).await {
                Ok(_) => {
                    log::info!("{} | Joined game with id {}", trace_id.to_string(), id);
                    Ok(status::Custom(
                        Status::Ok,
                        Json(JoinGame {
                            trace_id,
                            message: String::from("Joined the game."),
                        }),
                    ))
                }
                Err(err) => {
                    log::error!("{} | {}", trace_id.to_string(), err.message);
                    Err(status::Custom(
                        err.status_code,
                        Json(ErrorMessage {
                            trace_id,
                            error_message: err.message,
                            error_code: None,
                        }),
                    ))
                }
            }
        }
        Err(err) => {
            log::error!("{} | {}", trace_id.to_string(), err.message);
            Err(status::Custom(
                err.status_code,
                Json(ErrorMessage {
                    trace_id,
                    error_message: err.message,
                    error_code: None,
                }),
            ))
        }
    }
}

#[get("/games/<id>")]
pub(crate) async fn get_game_state(
    id: &str,
    db: &State<DbConnection>,
) -> Result<GetGameStatusResponse, ErrorResponse> {
    let trace_id = Uuid::new_v4();
    log::info!(
        "{} | Received get game status request for id: {}",
        trace_id.to_string(),
        id
    );
    match db.get_game(id).await {
        Ok(game) => {
            log::info!("{} | Received game status {:?}", trace_id, game.state);
            Ok(status::Custom(
                Status::Ok,
                Json(GetGameStatus {
                    trace_id,
                    game_status: game.state,
                }),
            ))
        }
        Err(err) => {
            log::error!("{} | {}", trace_id.to_string(), err.message);
            Err(status::Custom(
                err.status_code,
                Json(ErrorMessage {
                    trace_id,
                    error_message: err.message,
                    error_code: None,
                }),
            ))
        }
    }
}

#[put("/games/<id>/cancel")]
pub(crate) async fn cancel_game(
    id: &str,
    db: &State<DbConnection>,
) -> Result<CancelGameResponse, ErrorResponse> {
    let trace_id = Uuid::new_v4();
    log::info!(
        "{} | Received cancel game request for id: {}",
        trace_id.to_string(),
        id
    );
    match db.get_game(id).await {
        Ok(mut game) => match game.state {
            GameState::Pending | GameState::Ongoing => {
                game.state = GameState::Cancelled;
                match db.update_game(game).await {
                    Ok(_) => {
                        log::info!("{} | Cancelled the game with id {}", trace_id, id);
                        Ok(status::Custom(Status::Ok, Json(CancelGame { trace_id })))
                    }
                    Err(err) => {
                        log::error!("{} | {}", trace_id.to_string(), err.message);
                        Err(status::Custom(
                            err.status_code,
                            Json(ErrorMessage {
                                trace_id,
                                error_message: err.message,
                                error_code: None,
                            }),
                        ))
                    }
                }
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
        Err(err) => {
            log::error!("{} | {}", trace_id.to_string(), err.message);
            Err(status::Custom(
                err.status_code,
                Json(ErrorMessage {
                    trace_id,
                    error_message: err.message,
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

    const SURREALDB_VERSION: &str = "v2.0.4";

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
                "localhost:{}",
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
        let created_game: Game = db
            .conn
            .create("games")
            .content(game)
            .await
            .expect("Creating game failed.")
            .expect("");

        let response = client
            .put(uri!(super::join_game(created_game.id.id.to_string())))
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
        let created_game: Game = db
            .conn
            .create("games")
            .content(game)
            .await
            .expect("Creating game failed.")
            .expect("");

        let response = client
            .put(uri!(super::join_game(created_game.id.id.to_string())))
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
            let expected_state = game.state.clone();
            let created_game: Game = db
                .conn
                .create("games")
                .content(game)
                .await
                .expect("Creating game failed.")
                .expect("");
            let response = client
                .get(uri!(super::get_game_state(created_game.id.id.to_string())))
                .dispatch()
                .await;

            assert_eq!(response.status(), Status::Ok);
            let response = response
                .into_json::<responses::GetGameStatus>()
                .await
                .expect("Invalid response from server.");
            assert_eq!(response.game_status, expected_state);
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
            let created_game: Game = db
                .conn
                .create("games")
                .content(game)
                .await
                .expect("Creating game failed.")
                .expect("");
            let response = client
                .put(uri!(super::cancel_game(created_game.id.id.to_string())))
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
            let created_game: Game = db
                .conn
                .create("games")
                .content(game)
                .await
                .expect("Creating game failed.")
                .expect("");
            let response = client
                .put(uri!(super::cancel_game(created_game.id.id.to_string())))
                .dispatch()
                .await;

            assert_eq!(response.status(), Status::Conflict);
        }
    }
}

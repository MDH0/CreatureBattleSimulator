#[macro_use]
extern crate rocket;

use rocket::futures::future::err;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::serde_json::json;
use rocket::serde::json::Json;
use rocket::State;
use serde::de::value::StringDeserializer;
use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::Client;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::sql::{Id, Thing};
use surrealdb::Surreal;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct Game {
    id: Thing,
}

impl Default for Game {
    fn default() -> Self {
        Game {
            id: Thing::from(("games", Id::rand())),
        }
    }
}

struct DbConnection {
    pub conn: Surreal<Client>,
}

impl DbConnection {
    pub async fn init(url: &str, username: &str, password: &str) -> Result<Self, surrealdb::Error> {
        let db = Surreal::new::<Ws>(url).await?;

        db.signin(Root { username, password }).await?;

        db.use_ns("CreatureBattleSimulator")
            .use_db("CreatureBattleSimulator")
            .await?;

        Ok(DbConnection { conn: db })
    }
}

#[derive(Serialize, Deserialize)]
struct PostGameResponse {
    game_id: String,
}

#[post("/game")]
async fn create_game(
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
                ))
            }
            let game_id = result.get(0).unwrap().id.to_string();
            Ok(Json(PostGameResponse { game_id }))
        }
        Err(err) => Err(status::Custom(Status::InternalServerError, err.to_string())),
    }
}

#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
struct Config {
    username: String,
    password: String,
    db_url: String,
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let rocket = rocket::build();
    let figment = rocket.figment();
    let config: Config = figment.extract().expect("Panic");

    rocket
        .manage(
            DbConnection::init(
                config.db_url.as_str(),
                config.username.as_str(),
                config.password.as_str(),
            )
            .await
            .unwrap(),
        )
        .mount("/", routes![create_game])
        .launch()
        .await?;

    Ok(())
}

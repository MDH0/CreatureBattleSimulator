#[macro_use]
extern crate rocket;

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::Client;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct Game {
    pub game_id: Uuid,
}

impl Default for Game {
    fn default() -> Self {
        Game {
            game_id: Uuid::new_v4(),
        }
    }
}

struct DbConnection {
    pub conn: Surreal<Client>,
}

impl DbConnection {
    pub async fn init(url: &str, username: &str, password: &str) -> Result<Self, surrealdb::Error> {
        let db = Surreal::new::<Ws>(url).await?;

        db.signin(Root {
            username,
            password,
        })
        .await?;

        db.use_ns("CreatureBattleSimulator")
            .use_db("CreatureBattleSimulator")
            .await?;

        Ok(DbConnection { conn: db })
    }
}

#[post("/game")]
fn create_game() -> Json<Game> {
    let game = Game::default();
    Json(game)
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


    rocket.manage(DbConnection::init(config.db_url.as_str(), config.username.as_str(), config.password.as_str()).await.unwrap())
        .mount("/", routes![create_game])
        .launch()
        .await?;

    Ok(())
}

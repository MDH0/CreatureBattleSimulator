mod responses;
mod routes;
mod db;

#[macro_use]
extern crate rocket;

use serde::{Deserialize, Serialize};
use crate::db::DbConnection;

#[derive(Serialize, Deserialize)]
enum GameState {
    Pending,
    Started,
    Finished,
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
        .mount("/", routes![routes::create_game])
        .launch()
        .await?;

    Ok(())
}

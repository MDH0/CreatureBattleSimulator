mod api;
mod db;

#[macro_use]
extern crate rocket;

use crate::{api::routes, db::DbConnection};
use rocket::{Build, Rocket};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
struct Config {
    username: String,
    password: String,
    db_url: String,
}

async fn build_the_rocket() -> Rocket<Build> {
    let rocket = rocket::build();
    let figment = rocket.figment();
    let config: Config = figment
        .extract()
        .expect("Missing username, password or database url.");

    rocket
        .manage(
            //Can we directly manage the underlying Surreal<Client> and use a helper function instead?
            DbConnection::init(
                config.db_url.as_str(),
                config.username.as_str(),
                config.password.as_str(),
            )
            .await
            .unwrap(),
        )
        .mount("/", routes![routes::create_game, routes::join_game, routes::get_game_state])
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _ = build_the_rocket().await.launch().await?;

    Ok(())
}

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
    pub username: String,
    pub password: String,
    pub db_url: String,
}

async fn build_the_rocket(rocket: Rocket<Build>, config: Config) -> Rocket<Build> {
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
        .mount(
            "/",
            routes![
                routes::create_game,
                routes::join_game,
                routes::get_game_state
            ],
        )
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    let rocket = rocket::build();
    let figment = rocket.figment();
    let config = figment
        .extract()
        .expect("Missing username, password or database url.");

    let _ = build_the_rocket(rocket, config).await.launch().await?;

    Ok(())
}

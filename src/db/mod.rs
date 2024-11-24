pub mod entities;
use crate::db::entities::Game;
use rocket::http::Status;
#[cfg(test)]
use surrealdb::opt::auth::Root;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    Surreal,
};

#[cfg(not(test))]
use surrealdb::opt::auth::Namespace;

pub struct DbConnection {
    pub conn: Surreal<Client>,
}

impl DbConnection {
    //Very hack. Need to execute the method differently, because when starting the testcontainer, you can't log into a non-existent namespace.
    //Apparently got added somewhere after SurrealDB 1.5.4.
    #[cfg(not(test))]
    pub async fn init(url: &str, username: &str, password: &str) -> Result<Self, surrealdb::Error> {
        let db = Surreal::new::<Ws>(url).await?;

        db.signin(Namespace {
            namespace: "CreatureBattleSimulator",
            username,
            password,
        })
        .await?;

        db.use_db("CreatureBattleSimulator").await?;

        Ok(DbConnection { conn: db })
    }

    #[cfg(test)]
    pub async fn init(url: &str, username: &str, password: &str) -> Result<Self, surrealdb::Error> {
        let db = Surreal::new::<Ws>(url).await?;

        db.signin(Root { username, password }).await?;

        db.use_ns("CreatureBattleSimulator")
            .use_db("CreatureBattleSimulator")
            .await?;

        Ok(DbConnection { conn: db })
    }

    pub async fn create_game(&self) -> Result<String, DbError> {
        let game = Game::default();
        let query_result: Option<Game> = self.conn.create("games").content(game).await?;
        match query_result {
            None => Err(DbError {
                message: String::from("Couldn't create the lobby"),
                status_code: Status::InternalServerError,
            }),
            Some(created_game) => Ok(created_game.id.id.to_string()),
        }
    }

    pub async fn get_game(&self, game_id: &str) -> Result<Game, DbError> {
        let query_result: Option<Game> = self.conn.select(("games", game_id)).await?;
        match query_result {
            None => Err(DbError {
                message: String::from("Couldn't find the game you're looking for."),
                status_code: Status::NotFound,
            }),
            Some(game) => Ok(game),
        }
    }

    pub async fn update_game(&self, updated_game: Game) -> Result<(), DbError> {
        let update_result: Option<Game> = self
            .conn
            .update(("games", updated_game.id.id.to_string()))
            .content(updated_game)
            .await?;
        match update_result {
            None => Err(DbError {
                message: String::from("Couldn't find the game you're looking for."),
                status_code: Status::NotFound,
            }),
            Some(_) => Ok(()),
        }
    }
}

pub struct DbError {
    pub message: String,
    pub status_code: Status,
}

impl From<surrealdb::Error> for DbError {
    fn from(value: surrealdb::Error) -> Self {
        DbError {
            message: value.to_string(),
            status_code: Status::InternalServerError,
        }
    }
}

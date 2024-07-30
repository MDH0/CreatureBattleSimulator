use crate::GameState;
use rocket::serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::sql::{Id, Thing};
use surrealdb::Surreal;

#[derive(Serialize, Deserialize)]
pub struct Game {
    pub(crate) id: Thing,
    state: GameState,
}

impl Default for Game {
    fn default() -> Self {
        Game {
            id: Thing::from(("games", Id::rand())),
            state: GameState::Pending,
        }
    }
}

pub struct DbConnection {
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

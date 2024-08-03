pub mod entities;

use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Namespace,
    Surreal,
};

pub struct DbConnection {
    pub conn: Surreal<Client>,
}

impl DbConnection {
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
}

use crate::api::lobbies::*;
use rocket::Route;

mod lobbies;
pub mod responses;

pub fn get_routes() -> Vec<Route> {
    routes![create_game, join_game, get_game_state, cancel_game]
}

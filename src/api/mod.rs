use rocket::Route;
use crate::api::lobbies::*;

pub mod responses;
mod lobbies;

pub fn get_routes() -> Vec<Route> {
    routes![create_game, join_game, get_game_state, cancel_game]
}
#[macro_use] extern crate rocket;

mod routes;
mod auth;
mod db;
mod state;
mod models;

use crate::db::{init_db, init_pool};
use crate::state::AppState;
use rocket_dyn_templates::Template;

#[launch]
fn rocket() -> _ {
    let pool = init_pool("winter_card.db");
    init_db(&pool);
    
    let state = AppState::new(pool);

    rocket::build()
        .manage(state)
        .mount("/", routes::routes())
        .attach(Template::fairing())
}
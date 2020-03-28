#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate diesel;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate rocket;
#[macro_use] extern crate validator_derive;
extern crate validator;
extern crate rocket_contrib;
extern crate dotenv;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate chrono;
extern crate bcrypt;
extern crate time;

mod models;
mod database;
mod repository;
mod schema;
mod controllers;

use dotenv::dotenv;
use rocket_cors::{AllowedOrigins, Error};
use rocket::http::Method::{Get, Post};

fn main() -> Result<(), Error> {
    dotenv().ok();
    let allowed_origins = AllowedOrigins::some_exact(&["http://localhost:4200"]);
    let cors = rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods: vec![Get, Post].into_iter().map(From::from).collect(),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()?;
    let routes = routes![
        controllers::users::register_user,
        controllers::users::login_user,
        controllers::users::authenticate
    ];
    rocket::ignite()
        .mount("/", routes)
        .attach(cors)
        .launch();
    Ok(())
}

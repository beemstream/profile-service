#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate diesel;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate rocket;
#[macro_use] extern crate validator_derive;
extern crate serde;
extern crate serde_json;
extern crate validator;
extern crate rocket_contrib;
extern crate dotenv;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate chrono;
extern crate bcrypt;
extern crate time;
extern crate base64;
extern crate oauth2;
extern crate rand;
extern crate url;

mod models;
mod database;
mod repository;
mod schema;
mod controllers;
mod oauth;
mod jwt;
mod util;

use dotenv::dotenv;
use rocket_cors::{AllowedOrigins, Error};
use rocket::http::Method::{Get, Post};

fn setup_up_cors() -> Result<rocket_cors::Cors, Error> {
    let allowed_origins_env = std::env::var("ALLOWED_ORIGINS").expect("No origins set");
    let origins: Vec<&str> = allowed_origins_env.split(",").collect();
    let allowed_origins = AllowedOrigins::some_exact(origins.as_slice());

    rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods: vec![Get, Post].into_iter().map(From::from).collect(),
        allow_credentials: true,
        ..Default::default()
    }.to_cors()
}

fn main() -> Result<(), Error> {
    dotenv().ok();
    let routes = routes![
        controllers::users::register_user,
        controllers::users::login_user,
        controllers::users::authenticate,
        controllers::oauth::twitch_auth,
        controllers::oauth::twitch_token
    ];
    rocket::ignite()
        .mount("/", routes)
        .attach(setup_up_cors()?)
        .launch();
    Ok(())
}

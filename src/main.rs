#[macro_use]
extern crate diesel;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate validator_derive;
extern crate base64;
extern crate chrono;
extern crate dotenv;
extern crate oauth2;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rand;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;
extern crate time;
extern crate url;
extern crate validator;

mod controllers;
mod database;
mod jwt;
mod models;
mod oauth;
mod repository;
mod schema;
mod util;

#[cfg(test)]
mod test;

use dotenv::dotenv;
use rocket::{
    http::Method::{Get, Post},
    Rocket, Route,
};
use rocket_cors::{AllowedOrigins, Error};

fn setup_up_cors() -> Result<rocket_cors::Cors, Error> {
    let origins: Vec<&str> = util::globals::ALLOWED_ORIGINS.split(",").collect();
    let allowed_origins = AllowedOrigins::some_exact(origins.as_slice());

    rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods: vec![Get, Post].into_iter().map(From::from).collect(),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
}

#[launch]
fn get_rocket() -> Rocket {
    dotenv().ok();
    let routes: Vec<Route> = routes![
        controllers::users::register_user,
        controllers::users::login_user,
        controllers::users::refresh_token,
        controllers::users::authenticate,
        controllers::oauth::twitch_auth,
        controllers::oauth::twitch_token,
    ];
    rocket::ignite()
        .mount("/", routes)
        .attach(setup_up_cors().unwrap())
}

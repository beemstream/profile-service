#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate validator_derive;
extern crate futures;
extern crate serde;
extern crate serde_json;
extern crate validator;
#[macro_use]
extern crate rocket_contrib;
extern crate argon2;
extern crate base64;
extern crate chrono;
extern crate oauth2;
extern crate rand;
extern crate time;
extern crate url;

mod controllers;
mod database;
mod email_sender;
mod jwt;
mod models;
mod oauth;
mod repository;
mod schema;
mod util;

#[cfg(test)]
mod test;

use database::DbConn;
use jwt::jwt_validation;
use rocket::{
    http::Method::{Get, Post},
    Rocket, Route,
};
use rocket_cors::{AllowedOrigins, Error};
use util::globals::{GlobalConfig, JWTConfig, TwitchConfig};

fn setup_up_cors(origins: &Vec<String>) -> Result<rocket_cors::Cors, Error> {
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
    let rocket = rocket::ignite();
    let routes: Vec<Route> = routes![
        controllers::users::register_user,
        controllers::users::login_user,
        controllers::users::refresh_token,
        controllers::users::authenticate,
        controllers::oauth::twitch_auth,
        controllers::oauth::twitch_token,
    ];

    let figment = rocket.figment();

    let global_config: GlobalConfig = figment.extract().expect("global config");
    let twitch_config: TwitchConfig = figment.extract().expect("twitch config");

    let jwt = JWTConfig {
        validation: jwt_validation(),
    };

    rocket
        .mount("/", routes)
        .attach(DbConn::fairing())
        .attach(setup_up_cors(&global_config.allowed_origins).unwrap())
        .manage(global_config)
        .manage(jwt)
        .manage(twitch_config)
}

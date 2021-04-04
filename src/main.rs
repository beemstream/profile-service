#[macro_use]
extern crate diesel;

mod database;
mod email_sender;
mod jwt;
mod models;
mod oauth;
mod repository;
mod routes;
mod schema;
mod util;

#[cfg(test)]
mod test;

use database::DbConn;
use jwt::jwt_validation;
use rocket::{
    catch, catchers,
    http::Method::{Get, Post},
    launch, routes, Request, Rocket, Route,
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

#[catch(401)]
fn not_authorized(_req: &Request) {
    ()
}

#[launch]
fn get_rocket() -> Rocket {
    openssl_probe::init_ssl_cert_env_vars();
    let rocket = rocket::ignite();
    let routes: Vec<Route> = routes![
        routes::register::register_user,
        routes::login::login,
        routes::refresh_token::refresh_token,
        routes::users::authenticate,
        routes::oauth::twitch_auth,
        routes::oauth::twitch_token,
        routes::profile_lookup::profile_lookup
    ];

    let figment = rocket.figment();

    let global_config: GlobalConfig = figment.extract().expect("global config");
    let twitch_config: TwitchConfig = figment.extract().expect("twitch config");
    let jwt = JWTConfig {
        validation: jwt_validation(),
    };

    rocket
        .mount("/auth", routes)
        .attach(DbConn::fairing())
        .attach(setup_up_cors(&global_config.allowed_origins).unwrap())
        .manage(global_config)
        .manage(jwt)
        .manage(twitch_config)
        .register(catchers![not_authorized])
}

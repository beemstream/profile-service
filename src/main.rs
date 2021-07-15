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
    launch, routes, Build, Request, Rocket, Route,
};
use util::globals::{EmailConfig, GlobalConfig, JWTConfig, TwitchConfig};

#[catch(401)]
fn not_authorized(_req: &Request) {}

#[launch]
fn get_rocket() -> Rocket<Build> {
    let rocket = rocket::build();
    let routes: Vec<Route> = routes![
        routes::register::register_user,
        routes::login::login,
        routes::refresh_token::refresh_token,
        routes::users::authenticate,
        routes::oauth::twitch_auth,
        routes::oauth::twitch_token,
        routes::oauth::logout_twitch,
        routes::profile_lookup::profile_lookup,
    ];

    let figment = rocket.figment();

    let global_config: GlobalConfig = figment.extract().expect("global config");
    let twitch_config: TwitchConfig = figment.extract().expect("twitch config");
    let email_config: EmailConfig = figment.extract().expect("email config");
    let jwt = JWTConfig {
        validation: jwt_validation(),
    };

    rocket
        .mount("/auth", routes)
        .attach(DbConn::fairing())
        .manage(global_config)
        .manage(twitch_config)
        .manage(email_config)
        .manage(jwt)
        .register("/", catchers![not_authorized])
}

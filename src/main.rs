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
use rocket::{fairing::{Info, Fairing, Kind}, http::Method::{Get, Post}};
use rocket_cors::{Error, AllowedOrigins};
use std::time::SystemTime;

fn setup_up_cors() -> Result<rocket_cors::Cors, Error> {
    let origins: Vec<&str> = util::globals::ALLOWED_ORIGINS.split(",").collect();
    let allowed_origins = AllowedOrigins::some_exact(origins.as_slice());

    rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods: vec![Get, Post].into_iter().map(From::from).collect(),
        allow_credentials: true,
        ..Default::default()
    }.to_cors()
}

struct MiddleWare;

impl Fairing for MiddleWare {
    fn on_attach(&self, rocket: rocket::Rocket) -> Result<rocket::Rocket, rocket::Rocket> { Ok(rocket) }
    fn on_launch(&self, _rocket: &rocket::Rocket) {}
    fn on_request(&self, request: &mut rocket::Request, _data: &rocket::Data) {
        request.local_cache(|| TimerStart(Some(SystemTime::now())));
    }
    fn on_response(&self, request: &rocket::Request, _response: &mut rocket::Response) {
        let start_time = request.local_cache(|| TimerStart(None));
        if let Some(Ok(duration)) = start_time.0.map(|st| st.elapsed()) {
            let ms = duration.as_secs() * 1000 + duration.subsec_millis() as u64;
            println!("{:?}:{:?}:{:?}ms", request.method(), request.uri().path(), ms);
        }
    }
    fn info(&self) -> rocket::fairing::Info {
        Info {
            kind: Kind::Request | Kind::Response,
            name: "Performance"
        }
    }

}

#[derive(Copy, Clone)]
struct TimerStart(Option<SystemTime>);


fn main() -> Result<(), Error> {
    dotenv().ok();
    let routes = routes![
        controllers::users::register_user,
        controllers::users::login_user,
        controllers::users::refresh_token,
        controllers::users::authenticate,
        controllers::oauth::twitch_auth,
        controllers::oauth::twitch_token
    ];
    rocket::ignite()
        .mount("/", routes)
        .attach(setup_up_cors()?)
        .attach(MiddleWare)
        .launch();
    Ok(())
}

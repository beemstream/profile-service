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
use rocket::{fairing::{Info, Fairing, Kind}, http::Method::{Get, Post}, Rocket, Route};
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
        .attach(MiddleWare)
}

fn main() -> Result<(), Error> {
    dotenv().ok();
    get_rocket();
    Ok(())
}

#[cfg(test)]
mod test {
    use super::rocket;
    use rocket::local::Client;
    use rocket::http::{ContentType, Status, Header};
    use serde_json::Value;

    #[test]
    fn creates_user_successfully() {
        let client = Client::new(crate::get_rocket()).expect("valid rocket instance");
        let mut response = client
            .post("/register")
            .header(ContentType::JSON)
            .body(r#"{ "username": "ibrahim", "email": "ibrahim@gmail.com", "password": "Ibrahim123123" }"#)
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some("{\"status\":\"ok\"}".into()));
    }

    #[test]
    fn cannot_create_user_with_same_username() {
        let client = Client::new(crate::get_rocket()).expect("valid rocket instance");
        client
            .post("/register")
            .header(ContentType::JSON)
            .body(r#"{ "username": "foobar", "email": "foobar@gmail.com", "password": "Ibrahim123123" }"#)
            .dispatch();
        let mut response = client
            .post("/register")
            .header(ContentType::JSON)
            .body(r#"{ "username": "foobar", "email": "foobar@gmail.com", "password": "Ibrahim123123" }"#)
            .dispatch();
        assert_eq!(response.status(), Status::BadRequest);
        assert_eq!(response.body_string().unwrap().contains("Username already exists."), true);
    }

    #[test]
    fn cannot_create_user_with_not_strong_password() {
        let client = Client::new(crate::get_rocket()).expect("valid rocket instance");
        let mut response = client
            .post("/register")
            .header(ContentType::JSON)
            .body(r#"{ "username": "bazfoo", "email": "bazfoo@gmail.com", "password": "Ibrahim123" }"#)
            .dispatch();
        assert_eq!(response.status(), Status::BadRequest);
        assert_eq!(response.body_string().unwrap().contains("Password must be 12 characters or more"), true);
    }

    #[test]
    fn login_user_successfully() {
        let client = Client::new(crate::get_rocket()).expect("valid rocket instance");
        client
            .post("/register")
            .header(ContentType::JSON)
            .body(r#"{ "username": "bazfoo2", "email": "bazfoo2@gmail.com", "password": "Ibrahim123123" }"#)
            .dispatch();

        let mut response = client
            .post("/login")
            .header(ContentType::JSON)
            .body(r#"{ "identifier": "bazfoo2", "password": "Ibrahim123123" }"#)
            .dispatch();

        let body = response.body_string().unwrap();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(body.contains("\"status\":\"ok\""), true);
        assert_eq!(body.contains("\"access_token\""), true);
        assert_eq!(body.contains("\"expires_in\""), true);
    }

    #[test]
    fn authenticates_token_successfully() {
        let client = Client::new(crate::get_rocket()).expect("valid rocket instance");
        client
            .post("/register")
            .header(ContentType::JSON)
            .body(r#"{ "username": "bazfoo3", "email": "bazfoo3@gmail.com", "password": "Ibrahim123123" }"#)
            .dispatch();

        let mut token_response = client
            .post("/login")
            .header(ContentType::JSON)
            .body(r#"{ "identifier": "bazfoo3", "password": "Ibrahim123123" }"#)
            .dispatch();

        let token: Value = serde_json::from_str(token_response.body_string().unwrap().as_str()).unwrap();
        let access_token: String = token["access_token"].as_str().unwrap().to_owned();

        let mut request = client
            .get("/authenticate")
            .header(ContentType::JSON);

        request.add_header(Header::new("token", access_token));

        let response = request.dispatch();

        assert_eq!(response.status(), Status::Ok);
    }
}

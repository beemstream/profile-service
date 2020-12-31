use rocket::{
    http::{ContentType, Status},
    local::blocking::{Client, LocalResponse},
};
use serde_json::{json, Value};
use std::{
    panic,
    sync::{Mutex, MutexGuard},
};

mod authenticate;
mod login;
mod refresh_token;
mod register;

pub fn run_test<T>(test: T) -> ()
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    let result = panic::catch_unwind(|| test());
    assert!(result.is_ok())
}

pub fn get_access_token(body_string: &Option<String>) -> String {
    let token: Value = serde_json::from_str(body_string.clone().unwrap().as_str()).unwrap();
    token["access_token"].as_str().unwrap().to_owned()
}

pub fn create_user<'a>(client: &'a Client, username: &str) -> LocalResponse<'a> {
    let json = json!({
        "username": username,
        "email": format!("{}{}", username, "@gmail.com"),
        "password": "Ibrahim123123",
        "password_repeat": "Ibrahim123123"
    });

    let response = client
        .post("/register")
        .header(ContentType::JSON)
        .body(json.to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    response
}

lazy_static! {
    pub static ref ROCKET_CLIENT: Mutex<Client> =
        Mutex::new(Client::tracked(crate::get_rocket()).expect("valid rocket instance"));
}

pub fn get_client<'a>() -> MutexGuard<'a, Client> {
    ROCKET_CLIENT.lock().unwrap()
}

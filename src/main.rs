#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate diesel;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate rocket;
extern crate rocket_contrib;
extern crate dotenv;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate chrono;
extern crate bcrypt;

mod models;
mod database;
mod repository;
mod schema;
mod controllers;

use dotenv::dotenv;

fn main() {

    dotenv().ok();
    rocket::ignite().mount("/", routes![controllers::users::register_user]).launch();
}

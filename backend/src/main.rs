#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_sync_db_pools;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate diesel;
use rocket::fs::FileServer;

#[cfg(test)]
mod tests;

mod models;

mod fairings;

mod config;

mod mail;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(config::Config::init())
        .manage(mail::Mailer::new(false))
        .attach(fairings::Cors)
        .mount("/", FileServer::from("web"))
        .attach(models::stage())
}

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

#[get("/api/app-title")]
fn get_app_title() -> String {
    std::env::var("APP_TITLE").unwrap_or_else(|_| String::from("Tiny Tickets"))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(config::Config::init())
        .manage(mail::Mailer::new(
            std::env::var("TEST_MODE").unwrap_or_default() == "true",
        ))
        .attach(fairings::Cors)
        .mount("/", FileServer::from("web"))
        .mount("/", routes![get_app_title])
        .attach(models::stage())
}

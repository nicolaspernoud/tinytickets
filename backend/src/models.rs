use rocket::fairing::AdHoc;

pub mod asset;
pub mod comment;
pub mod db;
pub mod schema;
pub mod ticket;

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Models routes", |rocket| async {
        rocket
            .attach(db::stage())
            .attach(asset::stage())
            .attach(ticket::stage())
            .attach(comment::stage())
    })
}

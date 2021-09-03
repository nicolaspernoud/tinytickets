use crate::config::{AdminToken, UserToken};
use crate::mail::send_mail;
use crate::models::asset::Asset;
use crate::models::comment::Comment;
use crate::models::db::Db;
use crate::models::db::Result;
use crate::models::schema::*;
use handlebars::Context;
use handlebars::Handlebars;
use handlebars::Helper;
use handlebars::HelperResult;
use handlebars::Output;
use handlebars::RenderContext;
use handlebars::RenderError;
use rocket::data::{Data, ToByteUnit};
use rocket::fairing::AdHoc;
use rocket::response::status::Created;
use rocket::response::status::NotFound;
use rocket::response::Debug;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::tokio::fs::File;
use rocket::tokio::task::spawn_blocking;
use std::fs;

use rocket_sync_db_pools::diesel;

use self::diesel::prelude::*;

#[derive(
    Identifiable,
    Associations,
    Debug,
    Clone,
    Deserialize,
    Serialize,
    Queryable,
    Insertable,
    AsChangeset,
    PartialEq,
)]
#[serde(crate = "rocket::serde")]
#[belongs_to(Asset)]
#[table_name = "tickets"]
pub struct Ticket {
    pub id: i32,
    pub asset_id: i32,
    pub title: String,
    pub creator: String,
    pub description: String,
    pub time: chrono::NaiveDateTime,
    pub is_closed: bool,
}

#[derive(Clone, Insertable, Deserialize, Serialize, PartialEq, Debug)]
#[table_name = "tickets"]
pub struct InTicket {
    pub asset_id: i32,
    pub title: String,
    pub creator: String,
    pub description: String,
    pub time: chrono::NaiveDateTime,
    pub is_closed: bool,
}

impl PartialEq<InTicket> for Ticket {
    fn eq(&self, other: &InTicket) -> bool {
        self.title == other.title
            && self.asset_id == other.asset_id
            && self.creator == other.creator
            && self.description == other.description
            && self.time == other.time
    }
}

#[derive(Serialize)]
struct OutTicket {
    #[serde(flatten)]
    ticket: Ticket,
    comments: Vec<Comment>,
}

const PHOTOS_PATH: &str = "data/tickets/photos";

#[options("/<_..>")]
fn options() -> &'static str {
    ""
}

#[post("/", data = "<ticket>")]
async fn create(
    db: Db,
    ticket: Json<InTicket>,
    _token: UserToken<'_>,
) -> Result<Created<Json<Ticket>>, NotFound<String>> {
    let ticket_value = ticket.clone();
    let asset_id = ticket.asset_id;
    // Check that the asset that we want to create the ticket for exists...
    match db
        .run(move |conn| assets::table.find(asset_id).get_result::<Asset>(conn))
        .await
    {
        Ok(..) => {}
        Err(..) => {
            return Err(NotFound(
                "Cannot create ticket related to non existing asset.".to_string(),
            ));
        }
    }
    // ...create the ticket if so, and return the created ticket
    match db
        .run(move |conn| {
            if let Err(_err) = diesel::insert_into(tickets::table)
                .values(ticket_value)
                .execute(conn)
            {
                return Err(NotFound("Could not create ticket".to_string()));
            };

            match tickets::table
                .order(tickets::id.desc())
                .first::<Ticket>(conn)
            {
                Ok(r) => {
                    return Ok(r);
                }
                Err(..) => {
                    return Err(NotFound("Could not find ticket id".to_string()));
                }
            };
        })
        .await
    {
        Ok(t) => {
            let t2 = t.clone();
            spawn_blocking(move || match new_ticket_template(&t2) {
                Ok(r) => send_mail(r.0, r.1),
                Err(e) => println!("Handlebars error : {}", e),
            });
            return Ok(Created::new("/").body(Json(t)));
        }
        Err(e) => {
            return Err(e);
        }
    }
}

#[patch("/<id>", data = "<ticket>")]
async fn update(
    db: Db,
    ticket: Json<Ticket>,
    id: i32,
    _token: AdminToken<'_>,
) -> Result<Created<Json<Ticket>>> {
    let ticket_value = ticket.clone();
    db.run(move |conn| {
        diesel::update(tickets::table.filter(tickets::id.eq(id)))
            .set(ticket_value)
            .execute(conn)
    })
    .await?;

    Ok(Created::new("/").body(ticket))
}

#[get("/")]
async fn list(db: Db, _token: UserToken<'_>) -> Result<Json<Vec<i32>>> {
    let ids: Vec<i32> = db
        .run(|conn| tickets::table.select(tickets::id).load(conn))
        .await?;

    Ok(Json(ids))
}

#[get("/all")]
async fn list_all(db: Db, _token: UserToken<'_>) -> Result<Json<Vec<Ticket>>> {
    let all_tickets: Vec<Ticket> = db.run(|conn| tickets::table.load(conn)).await?;
    Ok(Json(all_tickets))
}

#[get("/mail_open")]
async fn mail_open(db: Db, _token: AdminToken<'_>) -> Result<Json<Vec<Ticket>>> {
    let open_tickets: Vec<Ticket> = db
        .run(|conn| {
            tickets::table
                .filter(tickets::is_closed.eq(false))
                .load(conn)
        })
        .await?;
    if open_tickets.len() > 0 {
        match open_tickets_template(&open_tickets) {
            Ok(r) => send_mail(r.0, r.1),
            Err(e) => println!("Handlebars error : {}", e),
        };
    };
    Ok(Json(open_tickets))
}

#[get("/<id>")]
async fn read(db: Db, id: i32, _token: UserToken<'_>) -> Result<Json<OutTicket>, NotFound<String>> {
    match db
        .run(move |conn| {
            let t: Result<Ticket, diesel::result::Error> =
                tickets::table.filter(tickets::id.eq(id)).first(conn);
            let t = match t {
                Ok(r) => r,
                Err(..) => {
                    return Err(NotFound("Could not get ticket".to_string()));
                }
            };
            let cs = <Comment>::belonging_to(&t).load(conn);
            let cs = match cs {
                Ok(r) => r,
                Err(..) => {
                    return Err(NotFound("Could not get comments for ticket".to_string()));
                }
            };
            Ok(OutTicket {
                ticket: t,
                comments: cs,
            })
        })
        .await
    {
        Ok(e) => Ok(Json(e)),
        Err(e) => {
            return Err(e);
        }
    }
}

#[delete("/<id>")]
async fn delete(db: Db, id: i32, _token: AdminToken<'_>) -> Result<Option<()>> {
    let affected = db
        .run(move |conn| {
            diesel::delete(tickets::table)
                .filter(tickets::id.eq(id))
                .execute(conn)
        })
        .await?;
    let filename = format!("{path}/{id}", path = PHOTOS_PATH, id = id);

    if let Err(e) = fs::remove_file(filename) {
        println!("error removing photo with id {}: {}", id, e);
    }

    Ok((affected == 1).then(|| ()))
}

#[delete("/")]
async fn destroy(db: Db, _token: AdminToken<'_>) -> Result<()> {
    db.run(move |conn| diesel::delete(tickets::table).execute(conn))
        .await?;
    Ok(())
}

#[post("/photos/<id>", data = "<image>")]
async fn upload(
    image: Data<'_>,
    _token: UserToken<'_>,
    id: i32,
) -> Result<String, Debug<std::io::Error>> {
    fs::create_dir_all(PHOTOS_PATH)?;
    let filename = format!("{path}/{id}", path = PHOTOS_PATH, id = id);
    image.open(10.mebibytes()).into_file(&filename).await?;
    Ok(filename)
}

#[get("/photos/<id>")]
async fn retrieve(id: i32, _token: UserToken<'_>) -> Result<File, NotFound<String>> {
    let filename = format!("{path}/{id}", path = PHOTOS_PATH, id = id);
    match File::open(&filename).await {
        Ok(f) => Ok(f),
        Err(..) => {
            return Err(NotFound("no image available".to_string()));
        }
    }
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Tickets routes", |rocket| async {
        rocket.mount(
            "/api/tickets",
            routes![
                options, list, list_all, read, create, update, delete, destroy, upload, retrieve,
                mail_open
            ],
        )
    })
}

fn new_ticket_template(t: &Ticket) -> Result<(String, String), RenderError> {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(".hbs", "templates")
        .expect("templates directory must exist!");

    match handlebars.render("new_ticket_subject", &t) {
        Ok(subject) => {
            match handlebars.render("new_ticket_body", &t) {
                Ok(body) => {
                    return Ok((subject, body));
                }
                Err(e) => return Err(e),
            };
        }
        Err(e) => return Err(e),
    };
}

fn open_tickets_template(t: &Vec<Ticket>) -> Result<(String, String), RenderError> {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(".hbs", "templates")
        .expect("templates directory must exist!");

    handlebars.register_helper("formattime", Box::new(formattime));

    match handlebars.render("open_tickets_subject", &t) {
        Ok(subject) => {
            match handlebars.render("open_tickets_body", &t) {
                Ok(body) => {
                    return Ok((subject, body));
                }
                Err(e) => return Err(e),
            };
        }
        Err(e) => return Err(e),
    };
}

fn formattime(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).unwrap();
    let value = param.value().clone();
    let t: Result<Ticket, serde_json::Error> = serde_json::from_value(value);
    match t {
        Ok(t) => {
            let time = t.time.format("%Y-%m-%d").to_string();
            out.write(&time)?;
            Ok(())
        }
        Err(..) => {
            out.write("[NOT A TICKET : CANNOT GET TIME]")?;
            Ok(())
        }
    }
}

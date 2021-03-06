use crate::{
    config::{AdminToken, Config, UserToken},
    mail::Mailer,
    models::{
        asset::Asset,
        comment::Comment,
        db::{Db, Result},
        schema::*,
    },
};
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext, RenderError};
use image::{imageops::FilterType::Lanczos3, GenericImageView};
use rocket::{
    data::{Data, ToByteUnit},
    fairing::AdHoc,
    response::{
        status::{Created, NotFound},
        Debug,
    },
    routes,
    serde::{json::Json, Deserialize, Serialize},
    tokio::{fs::File, task::spawn_blocking},
};
use std::fs;

use rocket_sync_db_pools::diesel;

use self::diesel::prelude::*;

macro_rules! trim {
    () => {
        fn trim(&mut self) -> &Self {
            self.title = self.title.trim().to_string();
            self.creator = self.creator.trim().to_string();
            self.creator_mail = self.creator_mail.trim().to_string();
            self.creator_phone = self.creator_phone.trim().to_string();
            self.description = self.description.trim().to_string();
            self
        }
    };
}

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
    pub creator_mail: String,
    pub creator_phone: String,
    pub description: String,
    pub time: chrono::NaiveDateTime,
    pub is_closed: bool,
}

impl Ticket {
    trim!();
}

#[derive(Clone, Insertable, Deserialize, Serialize, PartialEq, Debug)]
#[table_name = "tickets"]
pub struct InTicket {
    pub asset_id: i32,
    pub title: String,
    pub creator: String,
    pub creator_mail: String,
    pub creator_phone: String,
    pub description: String,
    pub time: chrono::NaiveDateTime,
    pub is_closed: bool,
}

impl InTicket {
    trim!();
}

impl PartialEq<InTicket> for Ticket {
    fn eq(&self, other: &InTicket) -> bool {
        self.title == other.title
            && self.asset_id == other.asset_id
            && self.creator == other.creator
            && self.creator_mail == other.creator_mail
            && self.creator_phone == other.creator_phone
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
    mut ticket: Json<InTicket>,
    _token: UserToken<'_>,
    mut mailer: Mailer,
    config: Config,
) -> Result<Created<Json<Ticket>>, NotFound<String>> {
    let ticket_value = ticket.trim().clone();
    let asset_id = ticket.asset_id;
    // Check that the asset that we want to create the ticket for exists...
    match db
        .run(move |conn| assets::table.find(asset_id).get_result::<Asset>(conn))
        .await
    {
        Ok(asset) => {
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
                        Ok(r) => Ok(r),
                        Err(..) => Err(NotFound("Could not find ticket id".to_string())),
                    }
                })
                .await
            {
                Ok(t) => {
                    let t2 = t.clone();
                    spawn_blocking(move || match template((asset, &t2), "new_ticket") {
                        Ok(r) => mailer.send_mail_to(r.0, r.1, config.ticket_mail_to),
                        Err(e) => println!("Handlebars error : {}", e),
                    });
                    Ok(Created::new("/").body(Json(t)))
                }
                Err(e) => Err(e),
            }
        }
        Err(..) => {
            return Err(NotFound(
                "Cannot create ticket related to non existing asset.".to_string(),
            ));
        }
    }
}

#[patch("/<id>", data = "<ticket>")]
async fn update(
    db: Db,
    mut ticket: Json<Ticket>,
    id: i32,
    _token: AdminToken<'_>,
    mut mailer: Mailer,
) -> Result<Created<Json<Ticket>>> {
    let t1 = ticket.trim().clone();
    db.run(move |conn| {
        diesel::update(tickets::table.filter(tickets::id.eq(id)))
            .set(t1)
            .execute(conn)
    })
    .await?;

    // If the ticket is closed, send a mail to the creator
    if ticket.is_closed {
        match ticket_with_comments(db, ticket.id).await {
            Ok(t) => {
                spawn_blocking(move || match template(&t, "closed_ticket") {
                    Ok(r) => mailer.send_mail_to(r.0, r.1, t.ticket.creator_mail),
                    Err(e) => println!("Handlebars error : {}", e),
                });
            }
            Err(e) => println!("{}", e),
        }
    }

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
    let all_tickets: Vec<Ticket> = db
        .run(|conn| tickets::table.order_by(tickets::time.desc()).load(conn))
        .await?;
    Ok(Json(all_tickets))
}

#[get("/mail_open")]
async fn mail_open(
    db: Db,
    _token: AdminToken<'_>,
    mut mailer: Mailer,
    config: Config,
) -> Result<Json<Vec<Ticket>>> {
    let open_tickets: Vec<Ticket> = db
        .run(|conn| {
            tickets::table
                .filter(tickets::is_closed.eq(false))
                .load(conn)
        })
        .await?;
    if !open_tickets.is_empty() {
        match template(&open_tickets, "open_tickets") {
            Ok(r) => mailer.send_mail_to(r.0, r.1, config.ticket_mail_to),
            Err(e) => println!("Handlebars error : {}", e),
        };
    };
    Ok(Json(open_tickets))
}

#[get("/<id>")]
async fn read(db: Db, id: i32, _token: UserToken<'_>) -> Result<Json<OutTicket>, NotFound<String>> {
    match ticket_with_comments(db, id).await {
        Ok(e) => Ok(Json(e)),
        Err(e) => Err(NotFound(e)),
    }
}

async fn ticket_with_comments(db: Db, id: i32) -> Result<OutTicket, String> {
    db.run(move |conn| {
        let t: Result<Ticket, diesel::result::Error> =
            tickets::table.filter(tickets::id.eq(id)).first(conn);
        let t = match t {
            Ok(r) => r,
            Err(..) => {
                return Err("Could not get ticket".to_string());
            }
        };
        let cs = <Comment>::belonging_to(&t).load(conn);
        let cs = match cs {
            Ok(r) => r,
            Err(..) => {
                return Err("Could not get comments for ticket".to_string());
            }
        };
        Ok(OutTicket {
            ticket: t,
            comments: cs,
        })
    })
    .await
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
    if let Err(e) = fs::remove_file(photo_filename(id)) {
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
    let filename = photo_filename(id);
    let img_bytes = image.open(10.mebibytes()).into_bytes().await?;
    match spawn_blocking(move || image::load_from_memory(&img_bytes)).await {
        Ok(r) => match r {
            Ok(r) => {
                match r
                    .resize(
                        std::cmp::min(1280, r.dimensions().0),
                        std::cmp::min(1280, r.dimensions().1),
                        Lanczos3,
                    )
                    .save_with_format(
                        &filename,
                        image::ImageFormat::from_extension("jpg").unwrap(),
                    ) {
                    Ok(_) => Ok(filename),
                    Err(_) => Err(Debug::from(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Error saving image",
                    ))),
                }
            }
            Err(_) => Err(Debug::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Error loading image",
            ))),
        },
        Err(_) => Err(Debug::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Error loading image",
        ))),
    }
}

#[get("/photos/<id>")]
async fn retrieve(id: i32, _token: UserToken<'_>) -> Result<File, NotFound<String>> {
    match File::open(photo_filename(id)).await {
        Ok(f) => Ok(f),
        Err(..) => Err(NotFound("no image available".to_string())),
    }
}

#[delete("/photos/<id>")]
async fn delete_photo(id: i32, _token: UserToken<'_>) -> Result<String, NotFound<String>> {
    match spawn_blocking(move || fs::remove_file(photo_filename(id))).await {
        Ok(..) => Ok("File deleted".to_string()),
        Err(..) => Err(NotFound("no image available".to_string())),
    }
}

fn photo_filename(id: i32) -> String {
    format!("{path}/{id}.jpg", path = PHOTOS_PATH, id = id)
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Tickets routes", |rocket| async {
        rocket.mount(
            "/api/tickets",
            routes![
                options,
                list,
                list_all,
                read,
                create,
                update,
                delete,
                destroy,
                upload,
                retrieve,
                delete_photo,
                mail_open
            ],
        )
    })
}

pub fn template<T>(o: T, template: &str) -> Result<(String, String), RenderError>
where
    T: Serialize,
{
    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(".hbs", "templates")
        .expect("templates directory must exist!");

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

    handlebars.register_helper("formattime", Box::new(formattime));

    match handlebars.render(format!("{}{}", template, "_body").as_str(), &o) {
        Ok(body) => {
            handlebars.register_escape_fn(handlebars::no_escape);
            let r_subject = handlebars.render(format!("{}{}", template, "_subject").as_str(), &o);
            match r_subject {
                Ok(subject) => Ok((subject, body)),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

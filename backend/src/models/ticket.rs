use crate::{
    config::{AdminToken, AppState, Config, UserToken},
    errors::internal_error,
    mail::Mailer,
    models::{asset::Asset, comment::Comment, schema::*},
};
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
    Json, Router,
};
use deadpool_diesel::{sqlite::Object, Manager};
use diesel::prelude::*;
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext, RenderError};
use image::{imageops::FilterType::Lanczos3, GenericImageView};
use serde::{Deserialize, Serialize};
use tokio::{fs::File, task::spawn_blocking};

use std::{fmt::Debug, fs};

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
    Selectable,
    Insertable,
    AsChangeset,
    PartialEq,
)]
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

pub fn build_tickets_router() -> Router {
    Router::new()
        .route("/api/tickets", get(list).post(create).delete(destroy))
        .route("/api/tickets/all", get(list_all))
        .route("/api/tickets/:id", patch(update).delete(delete).get(read))
        .route(
            "/api/tickets/photos/:id",
            post(upload).get(retrieve).delete(delete_photo),
        )
}

const PHOTOS_PATH: &str = "data/tickets/photos";

async fn create(
    State(pool): State<deadpool_diesel::sqlite::Pool>,
    mut ticket: Json<InTicket>,
    UserToken: UserToken,
    State(config): State<Config>,
) -> impl IntoResponse {
    let ticket_value = ticket.trim().clone();
    let asset_id = ticket.asset_id;
    // Check that the asset that we want to create the ticket for exists...
    let conn = pool.get().await.map_err(internal_error)?;

    match conn
        .run(move |conn| assets::table.find(asset_id).get_result::<Asset>(conn))
        .await
    {
        Ok(asset) => {
            // ...create the ticket if so, and return the created ticket
            let t: Ticket = conn
                .interact(|conn| {
                    diesel::insert_into(tickets::table)
                        .values(ticket_value)
                        .returning(Ticket::as_returning())
                        .get_result(conn)
                })
                .await
                .map_err(internal_error)?
                .map_err(internal_error)?;

            let t2 = &t;
            spawn_blocking(move || match template((asset, &t2), "new_ticket") {
                Ok(r) => state
                    .mailer
                    .send_mail_to(r.0, r.1, state.config.ticket_mail_to),
                Err(e) => println!("Handlebars error : {}", e),
            });
            Ok((StatusCode::CREATED, t))
        }
        Err(..) => Err((
            StatusCode::NOT_FOUND,
            "cannot create ticket related to non existing asset",
        )),
    }
}

async fn update(
    Path(id): Path<u32>,
    State(pool): State<deadpool_diesel::sqlite::Pool>,
    State(mailer): State<Mailer>,
    mut ticket: Json<Ticket>,
    AdminToken: AdminToken,
) -> impl IntoResponse {
    let t1 = ticket.trim().clone();
    let conn = pool.get().await.map_err(internal_error)?;

    conn.run(move |conn| {
        diesel::update(tickets::table.filter(tickets::id.eq(id)))
            .set(t1)
            .execute(conn)
    })
    .await?;

    // If the ticket is closed, send a mail to the creator
    if ticket.is_closed && !ticket.creator_mail.is_empty() {
        match ticket_with_comments(conn, ticket.id).await {
            Ok(t) => {
                spawn_blocking(move || match template(&t, "closed_ticket") {
                    Ok(r) => mailer.send_mail_to(r.0, r.1, t.ticket.creator_mail),
                    Err(e) => println!("Handlebars error : {}", e),
                });
            }
            Err(e) => println!("{}", e),
        }
    }

    Ok((StatusCode::CREATED, ticket))
}

async fn list(
    State(pool): State<deadpool_diesel::sqlite::Pool>,
    UserToken: UserToken,
) -> impl IntoResponse {
    let conn = pool.get().await.map_err(internal_error)?;

    let ids: Vec<i32> = conn
        .run(|conn| tickets::table.select(tickets::id).load(conn))
        .await?;

    Ok(Json(ids))
}

async fn list_all(
    State(pool): State<deadpool_diesel::sqlite::Pool>,
    UserToken: UserToken,
) -> impl IntoResponse {
    let conn = pool.get().await.map_err(internal_error)?;

    let all_tickets: Vec<Ticket> = conn
        .run(|conn| tickets::table.order_by(tickets::time.desc()).load(conn))
        .await?;
    Ok(Json(all_tickets))
}

async fn mail_open(
    State(pool): State<deadpool_diesel::sqlite::Pool>,
    UserToken: UserToken,
    State(mailer): State<Mailer>,
    State(config): State<Config>,
) -> impl IntoResponse {
    let conn = pool.get().await.map_err(internal_error)?;

    let open_tickets: Vec<Ticket> = conn
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

async fn read(
    State(pool): State<deadpool_diesel::sqlite::Pool>,
    id: i32,
    UserToken: UserToken,
) -> impl IntoResponse {
    let conn = pool.get().await.map_err(internal_error)?;

    match ticket_with_comments(conn, id).await {
        Ok(e) => Ok(Json(e)),
        Err(e) => Err(StatusCode::NOT_FOUND),
    }
}

async fn ticket_with_comments(db: Object, id: i32) -> Result<OutTicket, String> {
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

async fn delete(
    State(pool): State<deadpool_diesel::sqlite::Pool>,
    id: i32,
    AdminToken: AdminToken,
) -> impl IntoResponse {
    let conn = pool.get().await.map_err(internal_error)?;

    let affected = conn
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

async fn destroy(
    State(pool): State<deadpool_diesel::sqlite::Pool>,
    AdminToken: AdminToken,
) -> impl IntoResponse {
    let conn = pool.get().await.map_err(internal_error)?;
    conn.run(move |conn| diesel::delete(tickets::table).execute(conn))
        .await?;
    Ok(())
}

async fn upload(image: Bytes, UserToken: UserToken, id: i32) -> impl IntoResponse {
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

async fn retrieve(id: i32, UserToken: UserToken) -> impl IntoResponse {
    match File::open(photo_filename(id)).await {
        Ok(f) => Ok(f),
        Err(..) => Err((StatusCode::NOT_FOUND, "no image available")),
    }
}

async fn delete_photo(id: i32, UserToken: UserToken) -> impl IntoResponse {
    match spawn_blocking(move || fs::remove_file(photo_filename(id))).await {
        Ok(..) => Ok("File deleted".to_string()),
        Err(..) => Err((StatusCode::NOT_FOUND, "no image available")),
    }
}

fn photo_filename(id: i32) -> String {
    format!("{path}/{id}.jpg", path = PHOTOS_PATH, id = id)
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

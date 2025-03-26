use crate::{
    config::{AdminToken, AppState, Config, Db, UserToken},
    errors::ErrResponse,
    mail::Mailer,
    models::{asset::Asset, comment::Comment, schema::*},
};
use axum::{
    Json, Router,
    body::{Body, Bytes},
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, patch, post},
};
use deadpool_diesel::sqlite::Object;
use diesel::prelude::*;
use handlebars::{
    Context, DirectorySourceOptions, Handlebars, Helper, HelperResult, Output, RenderContext,
    RenderError,
};
use image::{GenericImageView, imageops::FilterType::Lanczos3};
use serde::{Deserialize, Serialize};
use serde_trim::string_trim;
use tokio::{fs::File, task::spawn_blocking};
use tokio_util::io::ReaderStream;

use std::{fmt::Debug, fs};

#[derive(
    Identifiable,
    Associations,
    Debug,
    Clone,
    Deserialize,
    Selectable,
    Serialize,
    Queryable,
    Insertable,
    AsChangeset,
    PartialEq,
)]
#[diesel(table_name = tickets, belongs_to(Asset))]
pub struct Ticket {
    pub id: i32,
    pub asset_id: i32,
    #[serde(deserialize_with = "string_trim")]
    pub title: String,
    #[serde(deserialize_with = "string_trim")]
    pub creator: String,
    #[serde(deserialize_with = "string_trim")]
    pub creator_mail: String,
    #[serde(deserialize_with = "string_trim")]
    pub creator_phone: String,
    #[serde(deserialize_with = "string_trim")]
    pub description: String,
    pub time: chrono::NaiveDateTime,
    pub is_closed: bool,
}

#[derive(Clone, Insertable, Deserialize, Serialize, PartialEq, Debug)]
#[diesel(table_name = tickets)]
pub struct InTicket {
    pub asset_id: i32,
    #[serde(deserialize_with = "string_trim")]
    pub title: String,
    #[serde(deserialize_with = "string_trim")]
    pub creator: String,
    #[serde(deserialize_with = "string_trim")]
    pub creator_mail: String,
    #[serde(deserialize_with = "string_trim")]
    pub creator_phone: String,
    #[serde(deserialize_with = "string_trim")]
    pub description: String,
    pub time: chrono::NaiveDateTime,
    pub is_closed: bool,
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

pub fn build_tickets_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create).delete(destroy))
        .route("/all", get(list_all))
        .route("/{id}", patch(update).delete(delete).get(read))
        .route(
            "/photos/{id}",
            post(upload).get(retrieve).delete(delete_photo),
        )
        .route("/mail_open", get(mail_open))
        .route("/export", get(export))
}

const PHOTOS_PATH: &str = "data/tickets/photos";

async fn create(
    State(mut mailer): State<Mailer>,
    State(config): State<Config>,
    _: UserToken,
    Db(db): Db,
    Json(ticket): Json<InTicket>,
) -> Result<(StatusCode, Json<Ticket>), ErrResponse> {
    let asset_id = ticket.asset_id;
    // Check that the asset that we want to create the ticket for exists...
    match db
        .interact(move |conn| assets::table.find(asset_id).get_result::<Asset>(conn))
        .await?
    {
        Ok(asset) => {
            // ...create the ticket if so, and return the created ticket
            let t = db
                .interact(|conn| {
                    diesel::insert_into(tickets::table)
                        .values(ticket)
                        .returning(Ticket::as_returning())
                        .get_result(conn)
                })
                .await??;

            let t2 = t.clone();
            spawn_blocking(move || match template((asset, &t), "new_ticket") {
                Ok(r) => mailer.send_mail_to(r.0, r.1, config.ticket_mail_to),
                Err(e) => println!("Handlebars error : {}", e),
            });
            Ok((StatusCode::CREATED, Json(t2)))
        }
        Err(..) => Err(ErrResponse::S404(
            "cannot create ticket related to non existing asset",
        )),
    }
}

async fn update(
    State(mut mailer): State<Mailer>,
    Path(id): Path<i32>,
    AdminToken: AdminToken,
    Db(db): Db,
    Json(ticket): Json<Ticket>,
) -> Result<StatusCode, ErrResponse> {
    let ticket = db
        .interact(move |conn| {
            diesel::update(tickets::table.filter(tickets::id.eq(id)))
                .set(ticket)
                .returning(Ticket::as_returning())
                .get_result(conn)
        })
        .await??;
    // If the ticket is closed, send a mail to the creator
    if ticket.is_closed && !ticket.creator_mail.is_empty() {
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
    Ok(StatusCode::NO_CONTENT)
}

async fn list(UserToken: UserToken, Db(db): Db) -> Result<impl IntoResponse, ErrResponse> {
    let res: Vec<i32> = db
        .interact(|conn| tickets::table.select(tickets::id).load(conn))
        .await??;
    Ok(Json(res))
}

async fn list_all(UserToken: UserToken, Db(db): Db) -> Result<impl IntoResponse, ErrResponse> {
    let all_tickets: Vec<Ticket> = db
        .interact(|conn| tickets::table.order(tickets::time.desc()).load(conn))
        .await??;
    Ok(Json(all_tickets))
}

async fn mail_open(
    Db(db): Db,
    UserToken: UserToken,
    State(mut mailer): State<Mailer>,
    State(config): State<Config>,
) -> Result<impl IntoResponse, ErrResponse> {
    let open_tickets: Vec<Ticket> = db
        .interact(|conn| {
            tickets::table
                .filter(tickets::is_closed.eq(false))
                .load(conn)
        })
        .await??;
    if !open_tickets.is_empty() {
        match template(&open_tickets, "open_tickets") {
            Ok(r) => mailer.send_mail_to(r.0, r.1, config.ticket_mail_to),
            Err(e) => println!("Handlebars error : {}", e),
        };
    };
    Ok(Json(open_tickets))
}

async fn export(Db(db): Db, UserToken: UserToken) -> Result<impl IntoResponse, ErrResponse> {
    let tickets_with_comments: Vec<(OutTicket, Asset)> = db
        .interact(|conn| -> Result<Vec<(OutTicket, Asset)>, ErrResponse> {
            let tickets_with_assets = tickets::table
                .inner_join(assets::table)
                .order(tickets::time.desc())
                .select((Ticket::as_select(), Asset::as_select()))
                .load::<(Ticket, Asset)>(conn)
                .map_err(|_| ErrResponse::S404("No tickets found"))?;

            let tickets = &tickets_with_assets
                .iter()
                .map(|e| &e.0)
                .collect::<Vec<&Ticket>>();

            let comments = Comment::belonging_to(tickets)
                .select(Comment::as_select())
                .order(comments::time.desc())
                .load(conn)
                .map_err(|_| ErrResponse::S404("No comments found"))?;

            let result = comments
                .grouped_by(tickets)
                .into_iter()
                .zip(tickets_with_assets)
                .map(|(cmts, t)| {
                    (
                        OutTicket {
                            ticket: t.0,
                            comments: cmts,
                        },
                        t.1,
                    )
                })
                .collect::<Vec<(OutTicket, Asset)>>();
            Ok(result)
        })
        .await??;

    match template(&tickets_with_comments, "tickets_with_comments") {
        Ok(r) => Ok(Html(r.1)),
        Err(_) => Err(ErrResponse::S500("could not export data")),
    }
}

async fn read(Db(db): Db, Path(id): Path<i32>, UserToken: UserToken) -> impl IntoResponse {
    match ticket_with_comments(db, id).await {
        Ok(e) => Ok(Json(e)),
        Err(e) => Err(e),
    }
}

async fn ticket_with_comments(db: Object, id: i32) -> Result<OutTicket, ErrResponse> {
    db.interact(move |conn| {
        let t: Result<Ticket, diesel::result::Error> =
            tickets::table.filter(tickets::id.eq(id)).first(conn);
        let t = match t {
            Ok(r) => r,
            Err(..) => {
                return Err(ErrResponse::S404("could not get ticket"));
            }
        };
        let cs = <Comment>::belonging_to(&t)
            .order(comments::time.desc())
            .load(conn);
        let cs = match cs {
            Ok(r) => r,
            Err(..) => {
                return Err(ErrResponse::S404("could not get comments for ticket"));
            }
        };
        Ok(OutTicket {
            ticket: t,
            comments: cs,
        })
    })
    .await?
}

async fn delete(
    Path(id): Path<i32>,
    AdminToken: AdminToken,
    Db(db): Db,
) -> Result<(), ErrResponse> {
    if db
        .interact(move |conn| {
            diesel::delete(tickets::table)
                .filter(tickets::id.eq(id))
                .execute(conn)
        })
        .await??
        == 1
    {
        if let Err(e) = fs::remove_file(photo_filename(id)) {
            println!("error removing photo with id {}: {}", id, e);
        }
        Ok(())
    } else {
        Err(ErrResponse::S404("object not found in database"))
    }
}

async fn destroy(AdminToken: AdminToken, Db(db): Db) -> Result<(), ErrResponse> {
    db.interact(move |conn| diesel::delete(tickets::table).execute(conn))
        .await??;
    Ok(())
}

async fn upload(
    UserToken: UserToken,
    Path(id): Path<i32>,
    image: Bytes,
) -> Result<String, ErrResponse> {
    fs::create_dir_all(PHOTOS_PATH)
        .map_err(|_| ErrResponse::S500("could not create images directory"))?;
    let filename = photo_filename(id);
    match spawn_blocking(move || image::load_from_memory(&image)).await {
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
                    Err(_) => Err(ErrResponse::S500("error saving image")),
                }
            }
            Err(_) => Err(ErrResponse::S500("error loading image")),
        },
        Err(_) => Err(ErrResponse::S500("error loading image")),
    }
}

async fn retrieve(
    Path(id): Path<i32>,
    UserToken: UserToken,
) -> Result<impl IntoResponse, ErrResponse> {
    let f = match File::open(photo_filename(id)).await {
        Ok(f) => f,
        Err(..) => {
            return Err(ErrResponse::S404("no image available"));
        }
    };
    let stream = ReaderStream::new(f);
    Ok(Body::from_stream(stream))
}

async fn delete_photo(Path(id): Path<i32>, UserToken: UserToken) -> impl IntoResponse {
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
        .register_templates_directory("templates", DirectorySourceOptions::default())
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
        let time = if let Ok(t) = t {
            Some(t.time.format("%Y-%m-%d").to_string())
        } else {
            let t: Result<Comment, serde_json::Error> =
                serde_json::from_value(param.value().clone());
            if let Ok(t) = t {
                Some(t.time.format("%Y-%m-%d").to_string())
            } else {
                None
            }
        };
        if let Some(time) = time {
            out.write(&time)?;
        } else {
            out.write("[NOT A TICKET NOR A COMMENT: CANNOT GET TIME]")?;
        }
        Ok(())
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

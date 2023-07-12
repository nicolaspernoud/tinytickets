use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch},
    Json, Router,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_trim::string_trim;
use tokio::task::spawn_blocking;

use crate::{
    config::{AdminToken, AppState, Config, Db, UserToken},
    errors::ErrResponse,
    mail::Mailer,
};

use super::{
    schema::{comments, tickets},
    ticket::Ticket,
};

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
    Selectable,
)]
#[diesel(table_name = comments, belongs_to(Ticket))]
pub struct Comment {
    pub id: i32,
    pub ticket_id: i32,
    pub time: chrono::NaiveDateTime,
    #[serde(deserialize_with = "string_trim")]
    pub creator: String,
    #[serde(deserialize_with = "string_trim")]
    pub content: String,
}

#[derive(Clone, Insertable, Deserialize, Serialize, PartialEq, Debug)]
#[diesel(table_name = comments)]
pub struct InComment {
    pub ticket_id: i32,
    pub time: chrono::NaiveDateTime,
    #[serde(deserialize_with = "string_trim")]
    pub creator: String,
    #[serde(deserialize_with = "string_trim")]
    pub content: String,
}

impl PartialEq<InComment> for Comment {
    fn eq(&self, other: &InComment) -> bool {
        self.ticket_id == other.ticket_id
            && self.content == other.content
            && self.time == other.time
            && self.creator == other.creator
    }
}

pub fn build_comments_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create).delete(destroy))
        .route("/all", get(list_all))
        .route("/:id", patch(update).delete(delete).get(read))
}

async fn create(
    State(mut mailer): State<Mailer>,
    State(config): State<Config>,
    _: UserToken,
    Db(db): Db,
    Json(comment): Json<InComment>,
) -> Result<(StatusCode, Json<Comment>), ErrResponse> {
    let ticket_id = comment.ticket_id;
    // Check that the ticket we want to create the comment for exists
    match db
        .interact(move |conn| tickets::table.find(ticket_id).get_result::<Ticket>(conn))
        .await?
    {
        Ok(ticket) => {
            // ...create the comment if so
            let c = comment.clone();
            match db
                .interact(move |conn| {
                    diesel::insert_into(comments::table)
                        .values(c)
                        .returning(Comment::as_returning())
                        .get_result(conn)
                })
                .await?
            {
                Ok(c) => {
                    spawn_blocking(move || {
                        match crate::models::ticket::template((&comment, &ticket), "new_comment") {
                            Ok(r) => mailer.send_mail_to(r.0, r.1, config.comment_mail_to),
                            Err(e) => println!("Handlebars error : {}", e),
                        }
                    });
                    Ok((StatusCode::CREATED, Json(c)))
                }

                Err(..) => Err(ErrResponse::S404("could not create comment")),
            }
        }
        Err(..) => Err(ErrResponse::S404(
            "cannot create comment related to non existing ticket",
        )),
    }
}

async fn update(
    Path(id): Path<i32>,
    AdminToken: AdminToken,
    Db(db): Db,
    Json(comment): Json<Comment>,
) -> Result<StatusCode, ErrResponse> {
    db.interact(move |conn| {
        diesel::update(comments::table.filter(comments::id.eq(id)))
            .set(comment)
            .execute(conn)
    })
    .await??;
    Ok(StatusCode::NO_CONTENT)
}

async fn list(UserToken: UserToken, Db(db): Db) -> Result<impl IntoResponse, ErrResponse> {
    let res: Vec<i32> = db
        .interact(|conn| comments::table.select(comments::id).load(conn))
        .await??;
    Ok(Json(res))
}

async fn list_all(UserToken: UserToken, Db(db): Db) -> Result<impl IntoResponse, ErrResponse> {
    let all_comments: Vec<Comment> = db.interact(|conn| comments::table.load(conn)).await??;
    Ok(Json(all_comments))
}

async fn read(
    Path(id): Path<i32>,
    UserToken: UserToken,
    Db(db): Db,
) -> Result<Json<Comment>, ErrResponse> {
    let comment: Comment = db
        .interact(move |conn| comments::table.filter(comments::id.eq(id)).first(conn))
        .await??;
    Ok(Json(comment))
}

async fn delete(
    Path(id): Path<i32>,
    AdminToken: AdminToken,
    Db(db): Db,
) -> Result<(), ErrResponse> {
    if db
        .interact(move |conn| {
            diesel::delete(comments::table)
                .filter(comments::id.eq(id))
                .execute(conn)
        })
        .await??
        == 1
    {
        Ok(())
    } else {
        Err(ErrResponse::S404("object not found in database"))
    }
}

async fn destroy(AdminToken: AdminToken, Db(db): Db) -> Result<(), ErrResponse> {
    db.interact(move |conn| diesel::delete(comments::table).execute(conn))
        .await??;
    Ok(())
}

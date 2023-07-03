use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch},
    Json, Router,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::task::spawn_blocking;

use crate::{
    config::{AdminToken, AppState, UserToken},
    errors::internal_error,
};

use super::{
    schema::{comments, tickets},
    ticket::Ticket,
};

macro_rules! trim {
    () => {
        fn trim(&mut self) -> &Self {
            self.creator = self.creator.trim().to_string();
            self.content = self.content.trim().to_string();
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
    Selectable,
    AsChangeset,
    PartialEq,
)]
#[belongs_to(Ticket)]
#[diesel(table_name = comments)]
pub struct Comment {
    pub id: i32,
    pub ticket_id: i32,
    pub time: chrono::NaiveDateTime,
    pub creator: String,
    pub content: String,
}

impl Comment {
    trim!();
}

#[derive(Clone, Insertable, Deserialize, Serialize, PartialEq, Debug)]
#[diesel(table_name = comments)]
pub struct InComment {
    pub ticket_id: i32,
    pub time: chrono::NaiveDateTime,
    pub creator: String,
    pub content: String,
}

impl InComment {
    trim!();
}

impl PartialEq<InComment> for Comment {
    fn eq(&self, other: &InComment) -> bool {
        self.ticket_id == other.ticket_id
            && self.content == other.content
            && self.time == other.time
            && self.creator == other.creator
    }
}

pub fn build_comments_router() -> Router {
    Router::new()
        .route("/api/comments", get(list).post(create).delete(destroy))
        .route("/api/comments/all", get(list_all))
        .route("/api/comments/:id", patch(update).delete(delete).get(read))
}

async fn create(
    State(pool): State<deadpool_diesel::sqlite::Pool>,
    mut comment: Json<InComment>,
    UserToken: UserToken,
    State(state): AppState,
) -> impl IntoResponse {
    let comment_value = comment.trim().clone();
    let ticket_id = comment.ticket_id;
    let conn = pool.get().await.map_err(internal_error)?;
    match conn
        .run(move |conn| tickets::table.find(ticket_id).get_result::<Ticket>(conn))
        .await
    {
        Ok(ticket) => {
            // ...create the comment if so
            match conn
                .run(move |conn| {
                    diesel::insert_into(comments::table)
                        .values(comment_value)
                        .execute(conn)
                })
                .await
            {
                Ok(..) => {
                    let c = (*comment).clone();
                    spawn_blocking(move || {
                        match crate::models::ticket::template((&c, &ticket), "new_comment") {
                            Ok(r) => {
                                state
                                    .mailer
                                    .send_mail_to(r.0, r.1, state.config.comment_mail_to)
                            }
                            Err(e) => println!("Handlebars error : {}", e),
                        }
                    });
                    Ok((StatusCode::CREATED, comment))
                }

                Err(..) => Err((StatusCode::NOT_FOUND, "could not create comment")),
            }
        }
        Err(..) => Err((
            StatusCode::NOT_FOUND,
            "Cannot create comment related to non existing ticket.",
        )),
    }
}

async fn update(
    Path(id): Path<u32>,
    State(pool): State<deadpool_diesel::sqlite::Pool>,
    mut comment: Json<Comment>,
    AdminToken: AdminToken,
) -> impl IntoResponse {
    let comment_value = comment.trim().clone();
    let conn = pool.get().await.map_err(internal_error)?;
    conn.run(move |conn| {
        diesel::update(comments::table.filter(comments::id.eq(id)))
            .set(comment_value)
            .execute(conn)
    })
    .await?;

    Ok((StatusCode::CREATED, comment))
}

async fn list(
    State(pool): State<deadpool_diesel::sqlite::Pool>,
    UserToken: UserToken,
) -> impl IntoResponse {
    let conn = pool.get().await.map_err(internal_error)?;

    let ids: Vec<i32> = conn
        .run(|conn| comments::table.select(comments::id).load(conn))
        .await?;

    Ok(Json(ids))
}

async fn list_all(
    State(pool): State<deadpool_diesel::sqlite::Pool>,
    UserToken: UserToken,
) -> impl IntoResponse {
    let conn = pool.get().await.map_err(internal_error)?;
    let all_comments: Vec<Comment> = conn.run(|conn| comments::table.load(conn)).await?;
    Ok(Json(all_comments))
}

async fn read(
    State(pool): State<deadpool_diesel::sqlite::Pool>,
    id: i32,
    UserToken: UserToken,
) -> impl IntoResponse {
    let conn = pool.get().await.map_err(internal_error)?;
    conn.run(move |conn| comments::table.filter(comments::id.eq(id)).first(conn))
        .await
        .map(Json)
        .ok()
}

async fn delete(
    State(pool): State<deadpool_diesel::sqlite::Pool>,
    id: i32,
    AdminToken: AdminToken,
) -> impl IntoResponse {
    let conn = pool.get().await.map_err(internal_error)?;
    let affected = conn
        .run(move |conn| {
            diesel::delete(comments::table)
                .filter(comments::id.eq(id))
                .execute(conn)
        })
        .await?;

    Ok((affected == 1).then(|| ()))
}

async fn destroy(
    State(pool): State<deadpool_diesel::sqlite::Pool>,
    AdminToken: AdminToken,
) -> impl IntoResponse {
    let conn = pool.get().await.map_err(internal_error)?;
    conn.run(move |conn| diesel::delete(comments::table).execute(conn))
        .await?;
    Ok(())
}

use self::diesel::prelude::*;
use crate::{
    config::{AdminToken, UserToken},
    models::{
        db::{Db, Result},
        schema::*,
        ticket::Ticket,
    },
};
use rocket::{
    fairing::AdHoc,
    response::status::{Created, NotFound},
    serde::{json::Json, Deserialize, Serialize},
};
use rocket_sync_db_pools::diesel;

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
#[belongs_to(Ticket)]
#[table_name = "comments"]
pub struct Comment {
    pub id: i32,
    pub ticket_id: i32,
    pub time: chrono::NaiveDateTime,
    pub creator: String,
    pub content: String,
}

#[derive(Clone, Insertable, Deserialize, Serialize, PartialEq, Debug)]
#[table_name = "comments"]
pub struct InComment {
    pub ticket_id: i32,
    pub time: chrono::NaiveDateTime,
    pub creator: String,
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

#[options("/<_..>")]
fn options() -> &'static str {
    ""
}

#[post("/", data = "<comment>")]
async fn create(
    db: Db,
    comment: Json<InComment>,
    _token: UserToken<'_>,
) -> Result<Created<Json<InComment>>, NotFound<String>> {
    let comment_value = comment.clone();
    let ticket_id = comment.ticket_id;
    // Check that the ticket that we want to create the comment for exists...
    match db
        .run(move |conn| tickets::table.find(ticket_id).get_result::<Ticket>(conn))
        .await
    {
        Ok(..) => {}
        Err(..) => {
            return Err(NotFound(
                "Cannot create comment related to non existing ticket.".to_string(),
            ));
        }
    }
    // ...create the comment if so
    match db
        .run(move |conn| {
            diesel::insert_into(comments::table)
                .values(comment_value)
                .execute(conn)
        })
        .await
    {
        Ok(..) => Ok(Created::new("/").body(comment)),
        Err(..) => Err(NotFound("Could not create comment".to_string())),
    }
}

#[patch("/<id>", data = "<comment>")]
async fn update(
    db: Db,
    comment: Json<Comment>,
    id: i32,
    _token: AdminToken<'_>,
) -> Result<Created<Json<Comment>>> {
    let comment_value = comment.clone();
    db.run(move |conn| {
        diesel::update(comments::table.filter(comments::id.eq(id)))
            .set(comment_value)
            .execute(conn)
    })
    .await?;

    Ok(Created::new("/").body(comment))
}

#[get("/")]
async fn list(db: Db, _token: UserToken<'_>) -> Result<Json<Vec<i32>>> {
    let ids: Vec<i32> = db
        .run(|conn| comments::table.select(comments::id).load(conn))
        .await?;

    Ok(Json(ids))
}

#[get("/all")]
async fn list_all(db: Db, _token: UserToken<'_>) -> Result<Json<Vec<Comment>>> {
    let all_comments: Vec<Comment> = db.run(|conn| comments::table.load(conn)).await?;
    Ok(Json(all_comments))
}

#[get("/<id>")]
async fn read(db: Db, id: i32, _token: UserToken<'_>) -> Option<Json<Comment>> {
    db.run(move |conn| comments::table.filter(comments::id.eq(id)).first(conn))
        .await
        .map(Json)
        .ok()
}

#[delete("/<id>")]
async fn delete(db: Db, id: i32, _token: AdminToken<'_>) -> Result<Option<()>> {
    let affected = db
        .run(move |conn| {
            diesel::delete(comments::table)
                .filter(comments::id.eq(id))
                .execute(conn)
        })
        .await?;

    Ok((affected == 1).then(|| ()))
}

#[delete("/")]
async fn destroy(db: Db, _token: AdminToken<'_>) -> Result<()> {
    db.run(move |conn| diesel::delete(comments::table).execute(conn))
        .await?;
    Ok(())
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Comments routes", |rocket| async {
        rocket.mount(
            "/api/comments",
            routes![options, list, list_all, read, create, update, delete, destroy],
        )
    })
}

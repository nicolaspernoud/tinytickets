use crate::config::AdminToken;
use crate::config::UserToken;
use crate::models::db::Db;
use crate::models::db::Result;
use crate::models::schema::*;
use rocket::fairing::AdHoc;
use rocket::response::status::Created;
use rocket::serde::{json::Json, Deserialize, Serialize};

use rocket_sync_db_pools::diesel;

use self::diesel::prelude::*;

#[derive(
    Identifiable,
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
#[table_name = "assets"]
pub struct Asset {
    pub id: i32,
    pub title: String,
    pub description: String,
}

#[derive(Clone, Insertable, Deserialize, Serialize, PartialEq, Debug)]
#[table_name = "assets"]
pub struct InAsset {
    pub title: String,
    pub description: String,
}

impl PartialEq<InAsset> for Asset {
    fn eq(&self, other: &InAsset) -> bool {
        self.title == other.title && self.description == other.description
    }
}

#[options("/<_..>")]
fn options() -> &'static str {
    ""
}

#[post("/", data = "<asset>")]
async fn create(
    db: Db,
    asset: Json<InAsset>,
    _token: AdminToken<'_>,
) -> Result<Created<Json<InAsset>>> {
    let asset_value = asset.clone();
    db.run(move |conn| {
        diesel::insert_into(assets::table)
            .values(asset_value)
            .execute(conn)
    })
    .await?;

    Ok(Created::new("/").body(asset))
}

#[patch("/<id>", data = "<asset>")]
async fn update(
    db: Db,
    asset: Json<Asset>,
    id: i32,
    _token: AdminToken<'_>,
) -> Result<Created<Json<Asset>>> {
    let asset_value = asset.clone();
    db.run(move |conn| {
        diesel::update(assets::table.filter(assets::id.eq(id)))
            .set(asset_value)
            .execute(conn)
    })
    .await?;

    Ok(Created::new("/").body(asset))
}

#[get("/")]
async fn list(db: Db, _token: UserToken<'_>) -> Result<Json<Vec<i32>>> {
    let ids: Vec<i32> = db
        .run(|conn| assets::table.select(assets::id).load(conn))
        .await?;

    Ok(Json(ids))
}

#[get("/all")]
async fn list_all(db: Db, _token: UserToken<'_>) -> Result<Json<Vec<Asset>>> {
    let all_assets: Vec<Asset> = db.run(|conn| assets::table.load(conn)).await?;
    Ok(Json(all_assets))
}

#[get("/<id>")]
async fn read(db: Db, id: i32, _token: UserToken<'_>) -> Option<Json<Asset>> {
    db.run(move |conn| assets::table.filter(assets::id.eq(id)).first(conn))
        .await
        .map(Json)
        .ok()
}

#[delete("/<id>")]
async fn delete(db: Db, id: i32, _token: AdminToken<'_>) -> Result<Option<()>> {
    let affected = db
        .run(move |conn| {
            diesel::delete(assets::table)
                .filter(assets::id.eq(id))
                .execute(conn)
        })
        .await?;

    Ok((affected == 1).then(|| ()))
}

#[delete("/")]
async fn destroy(db: Db, _token: AdminToken<'_>) -> Result<()> {
    db.run(move |conn| diesel::delete(assets::table).execute(conn))
        .await?;
    Ok(())
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Assets routes", |rocket| async {
        rocket.mount(
            "/api/assets",
            routes![options, list, list_all, read, create, update, delete, destroy],
        )
    })
}

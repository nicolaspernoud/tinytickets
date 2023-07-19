use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch},
    Json, Router,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_trim::string_trim;

use crate::{
    config::{AdminToken, AppState, Db, UserToken},
    errors::ErrResponse,
};

use super::schema::*;

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
    Selectable,
)]
#[diesel(table_name = assets)]
pub struct Asset {
    pub id: i32,
    #[serde(deserialize_with = "string_trim")]
    pub title: String,
    #[serde(deserialize_with = "string_trim")]
    pub description: String,
}

#[derive(Clone, Insertable, Deserialize, Serialize, PartialEq, Debug)]
#[diesel(table_name = assets)]
pub struct InAsset {
    #[serde(deserialize_with = "string_trim")]
    pub title: String,
    #[serde(deserialize_with = "string_trim")]
    pub description: String,
}

impl PartialEq<InAsset> for Asset {
    fn eq(&self, other: &InAsset) -> bool {
        self.title == other.title && self.description == other.description
    }
}

async fn create(
    _: AdminToken,
    Db(db): Db,
    Json(asset): Json<InAsset>,
) -> Result<(StatusCode, Json<Asset>), ErrResponse> {
    let asset = db
        .interact(|conn| {
            diesel::insert_into(assets::table)
                .values(asset)
                .returning(Asset::as_returning())
                .get_result(conn)
        })
        .await??;
    Ok((StatusCode::CREATED, Json(asset)))
}

async fn update(
    Path(id): Path<i32>,
    AdminToken: AdminToken,
    Db(db): Db,
    Json(asset): Json<Asset>,
) -> Result<StatusCode, ErrResponse> {
    db.interact(move |conn| {
        diesel::update(assets::table.filter(assets::id.eq(id)))
            .set(asset)
            .execute(conn)
    })
    .await??;
    Ok(StatusCode::NO_CONTENT)
}

async fn list(UserToken: UserToken, Db(db): Db) -> Result<impl IntoResponse, ErrResponse> {
    let res: Vec<i32> = db
        .interact(|conn| assets::table.select(assets::id).load(conn))
        .await??;
    Ok(Json(res))
}

async fn list_all(UserToken: UserToken, Db(db): Db) -> Result<impl IntoResponse, ErrResponse> {
    let all_assets: Vec<Asset> = db
        .interact(|conn| assets::table.order(assets::title).load(conn))
        .await??;
    Ok(Json(all_assets))
}

async fn read(
    Path(id): Path<i32>,
    UserToken: UserToken,
    Db(db): Db,
) -> Result<Json<Asset>, ErrResponse> {
    let asset: Asset = db
        .interact(move |conn| assets::table.filter(assets::id.eq(id)).first(conn))
        .await??;
    Ok(Json(asset))
}

async fn delete(
    Path(id): Path<i32>,
    AdminToken: AdminToken,
    Db(db): Db,
) -> Result<(), ErrResponse> {
    if db
        .interact(move |conn| {
            diesel::delete(assets::table)
                .filter(assets::id.eq(id))
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
    db.interact(move |conn| diesel::delete(assets::table).execute(conn))
        .await??;
    Ok(())
}

pub fn build_assets_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create).delete(destroy))
        .route("/all", get(list_all))
        .route("/:id", patch(update).delete(delete).get(read))
}

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch},
    Json, Router,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    config::{AdminToken, UserToken},
    errors::internal_error,
};

use super::schema::*;

// TODO : use validation or serde !
macro_rules! trim {
    () => {
        fn trim(&mut self) -> &Self {
            self.title = self.title.trim().to_string();
            self.description = self.description.trim().to_string();
            self
        }
    };
}

#[derive(
    Identifiable,
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
#[diesel(table_name = assets)]
pub struct Asset {
    pub id: i32,
    pub title: String,
    pub description: String,
}

impl Asset {
    trim!();
}

#[derive(Clone, Insertable, Deserialize, Serialize, PartialEq, Debug)]
#[table_name = "assets"]
pub struct InAsset {
    pub title: String,
    pub description: String,
}

impl InAsset {
    trim!();
}

impl PartialEq<InAsset> for Asset {
    fn eq(&self, other: &InAsset) -> bool {
        self.title == other.title && self.description == other.description
    }
}

async fn create(
    State(pool): State<deadpool_diesel::sqlite::Pool>,
    Json(mut asset): Json<InAsset>,
    AdminToken: AdminToken,
) -> Result<Json<InAsset>, (StatusCode, String)> {
    asset.trim();
    let conn = pool.get().await.map_err(internal_error)?;
    conn.interact(|conn| {
        diesel::insert_into(assets::table)
            .values(asset)
            .execute(conn)
    })
    .await
    .map_err(internal_error)?
    .map_err(internal_error)?;

    Ok(Json(asset))
}

async fn update(
    Path(id): Path<u32>,
    State(pool): State<deadpool_diesel::sqlite::Pool>,
    mut asset: Json<Asset>,
    AdminToken: AdminToken,
) -> impl IntoResponse {
    asset.trim();
    let conn = pool.get().await.map_err(internal_error)?;
    let res = conn
        .interact(|conn| {
            diesel::update(assets::table.filter(assets::id.eq(id)))
                .set(asset)
                .returning(Asset::as_returning())
                .get_result(conn)
        })
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;

    (StatusCode::OK, Json(res))
}

async fn list(
    State(pool): State<deadpool_diesel::sqlite::Pool>,
    UserToken: UserToken,
) -> impl IntoResponse {
    let conn = pool.get().await.map_err(internal_error)?;
    let res: Vec<i32> = conn
        .interact(|conn| assets::table.select(assets::id).load(conn))
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;
    Ok(Json(res))
}

async fn list_all(
    State(pool): State<deadpool_diesel::sqlite::Pool>,
    UserToken: UserToken,
) -> impl IntoResponse {
    let conn = pool.get().await.map_err(internal_error)?;
    let res = conn
        .interact(|conn| assets::table.select(Asset::as_select()).load(conn))
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;
    Ok(Json(res))
}

async fn read(
    State(pool): State<deadpool_diesel::sqlite::Pool>,
    id: i32,
    UserToken: UserToken,
) -> Option<Json<Asset>> {
    let conn = pool.get().await.map_err(internal_error)?;
    conn.run(move |conn| assets::table.filter(assets::id.eq(id)).first(conn))
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
            diesel::delete(assets::table)
                .filter(assets::id.eq(id))
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
    conn.run(move |conn| diesel::delete(assets::table).execute(conn))
        .await?;
    Ok(())
}

pub fn build_assets_router() -> Router {
    Router::new()
        .route("/api/assets", get(list).post(create).delete(destroy))
        .route("/api/assets/all", get(list_all))
        .route("/api/assets/:id", patch(update).delete(delete).get(read))
}

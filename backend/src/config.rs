use crate::mail::Mailer;
use axum::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::http::StatusCode;
use deadpool_diesel::sqlite::Manager;
use deadpool_diesel::{Pool, Runtime};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::env;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("db/migrations");

#[derive(Clone)]
pub(crate) struct AppState {
    config: Config,
    mailer: Mailer,
    pool: Pool<Manager>,
}

impl FromRef<AppState> for Config {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}

impl FromRef<AppState> for Mailer {
    fn from_ref(state: &AppState) -> Self {
        state.mailer.clone()
    }
}

impl FromRef<AppState> for Pool<Manager> {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

impl AppState {
    pub async fn new() -> Self {
        // set up connection pool
        let manager = Manager::new("db/db.sqlite", Runtime::Tokio1);
        let pool = Pool::builder(manager).max_size(8).build().unwrap();

        // run the migrations on server startup
        {
            let conn: deadpool_diesel::sqlite::Object = pool.get().await.unwrap();
            conn.interact(|conn| conn.run_pending_migrations(MIGRATIONS).map(|_| ()))
                .await
                .unwrap()
                .unwrap();
        }
        Self {
            config: Config::init(),
            mailer: Mailer::new(false),
            pool,
        }
    }
}

fn random_string() -> std::string::String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(48)
        .map(char::from)
        .collect()
}

#[derive(Clone)]
pub struct Config {
    admin_token: String,
    user_token: String,
    pub ticket_mail_to: String,
    pub comment_mail_to: String,
}

impl Config {
    pub fn init() -> Config {
        let admin_token = env::var("ADMIN_TOKEN").unwrap_or_else(|_| random_string());
        let user_token = env::var("USER_TOKEN").unwrap_or_else(|_| random_string());
        let ticket_mail_to = env::var("TICKET_MAIL_TO").unwrap_or_default();
        let comment_mail_to = env::var("COMMENT_MAIL_TO").unwrap_or_default();

        println!("Admin token is: {}", admin_token);
        println!("User token is: {}", user_token);

        Config {
            admin_token: format!("$ADMIN${}", admin_token),
            user_token: format!("$USER${}", user_token),
            ticket_mail_to,
            comment_mail_to,
        }
    }
}
pub struct AdminToken;

#[async_trait]
impl<S> FromRequestParts<S> for AdminToken
where
    S: Send + Sync,
    Config: FromRef<S>,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let config = Config::from_ref(state);
        if let Some(token) = parts.headers.get("X-TOKEN") {
            if token
                .to_str()
                .map_err(|_| (StatusCode::UNAUTHORIZED, "`X-TOKEN` header is corrupted"))?
                == config.admin_token
            {
                Ok(AdminToken)
            } else {
                Err((StatusCode::FORBIDDEN, "access denied"))
            }
        } else {
            Err((StatusCode::UNAUTHORIZED, "`X-TOKEN` header is missing"))
        }
    }
}

pub struct UserToken;

#[async_trait]
impl<S> FromRequestParts<S> for UserToken
where
    S: Send + Sync,
    Config: FromRef<S>,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let config = Config::from_ref(state);
        if let Some(token) = parts.headers.get("X-TOKEN") {
            if token
                .to_str()
                .map_err(|_| (StatusCode::UNAUTHORIZED, "`X-TOKEN` header is corrupted"))?
                == config.user_token
            {
                Ok(UserToken)
            } else {
                Err((StatusCode::FORBIDDEN, "access denied"))
            }
        } else {
            Err((StatusCode::UNAUTHORIZED, "`X-TOKEN` header is missing"))
        }
    }
}

pub struct Db(pub(crate) deadpool_diesel::sqlite::Object);

#[async_trait]
impl<S> FromRequestParts<S> for Db
where
    S: Send + Sync,
    Pool<Manager>: FromRef<S>,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match Pool::from_ref(state).get().await {
            Ok(db) => Ok(Db(db)),
            Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "database is unreachable")),
        }
    }
}

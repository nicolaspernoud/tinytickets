use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use std::env;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

fn random_string() -> std::string::String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(48)
        .map(char::from)
        .collect()
}

pub struct Config {
    admin_token: String,
    user_token: String,
}

impl Config {
    pub fn init() -> Config {
        let admin_t = env::var("ADMIN_TOKEN");
        let user_t = env::var("USER_TOKEN");
        let config = Config {
            admin_token: format!(
                "$ADMIN${}",
                if admin_t.is_err() {
                    random_string()
                } else {
                    admin_t.unwrap()
                }
            ),
            user_token: format!(
                "$USER${}",
                if user_t.is_err() {
                    random_string()
                } else {
                    user_t.unwrap()
                }
            ),
        };
        println!("Admin token is: {}", config.admin_token);
        println!("User token is: {}", config.user_token);
        config
    }
}

#[derive(Debug)]
pub enum TokenError {
    Missing,
    Invalid,
}

pub struct AdminToken<'r>(&'r str);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminToken<'r> {
    type Error = TokenError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        /// Returns true if `token` is a valid API token string.
        fn is_valid(token: &str, req: &Request<'_>) -> bool {
            match req.rocket().state::<Config>() {
                Some(config) => token == config.admin_token,
                None => false,
            }
        }

        match req.headers().get_one("X-TOKEN") {
            None => Outcome::Failure((Status::Unauthorized, TokenError::Missing)),
            Some(token) if is_valid(token, req) => Outcome::Success(AdminToken(token)),
            Some(_) => Outcome::Failure((Status::Forbidden, TokenError::Invalid)),
        }
    }
}

pub struct UserToken<'r>(&'r str);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserToken<'r> {
    type Error = TokenError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        /// Returns true if `token` is a valid API token string.
        fn is_valid(token: &str, req: &Request<'_>) -> bool {
            match req.rocket().state::<Config>() {
                Some(config) => token == config.user_token || token == config.admin_token,
                None => false,
            }
        }

        match req.headers().get_one("X-TOKEN") {
            None => Outcome::Failure((Status::Unauthorized, TokenError::Missing)),
            Some(token) if is_valid(token, req) => Outcome::Success(UserToken(token)),
            Some(_) => Outcome::Failure((Status::Forbidden, TokenError::Invalid)),
        }
    }
}

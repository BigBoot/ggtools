use rgcp_common::config::Config;
use rocket::{
    http::Status,
    request::{self, FromRequest, Request, State},
    Outcome::{self, Success},
};

pub struct ApiKey(String);

#[derive(Debug)]
pub enum ApiKeyError {
    InternalError,
    BadCount,
    Missing,
    Invalid,
}

impl<'a, 'r> FromRequest<'a, 'r> for ApiKey {
    type Error = ApiKeyError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        if let Success(config) = request.guard::<State<Config>>() {
            if config.api_key.is_none() {
                return Outcome::Failure((Status::BadRequest, ApiKeyError::Invalid));
            }
            let keys: Vec<_> = request.headers().get("x-api-key").collect();
            return match keys.len() {
                0 => Outcome::Failure((Status::BadRequest, ApiKeyError::Missing)),
                1 if Some(keys[0].to_owned()) == *config.api_key => Outcome::Success(ApiKey(keys[0].to_string())),
                1 => Outcome::Failure((Status::BadRequest, ApiKeyError::Invalid)),
                _ => Outcome::Failure((Status::BadRequest, ApiKeyError::BadCount)),
            };
        }

        return Outcome::Failure((Status::BadRequest, ApiKeyError::InternalError));
    }
}

use argonautica::{Hasher, Verifier};
use actix_session::Session;
use actix_web::{
  http::header::{CONTENT_TYPE, LOCATION}, 
  HttpRequest, 
  HttpResponse
};

use crate::{errors::AuthError, vars, models::SessionUser};


pub fn hash_password(password: &str) -> Result<String, AuthError> {
  Hasher::default()
      .with_password(password)
      .with_secret_key(vars::secret_key().as_str())
      .hash()
      .map_err(|_| AuthError::AuthenticationError(String::from("Could not hash password")))
}

pub fn verify(hash: &str, password: &str) -> Result<bool, AuthError> {
  Verifier::default()
      .with_hash(hash)
      .with_password(password)
      .with_secret_key(vars::secret_key().as_str())
      .verify()
      .map_err(|_| AuthError::AuthenticationError(String::from("Could not verify password")))
}

pub fn is_json_request(req: &HttpRequest) -> bool {
    req
      .headers()
      .get(CONTENT_TYPE)
      .map_or(
        false,
        |header| header.to_str().map_or(false, |content_type| "application/json" == content_type)
      )
}

pub fn is_signed_in(session: &Session) -> bool {
  match get_current_user(session) {
      Ok(_) => true,
      _ => false,
  }
}

pub fn set_current_user(session: &Session, user: &SessionUser) -> () {
    // serializing to string is alright for this case, 
    // but binary would be preferred in production use-cases.
    session.set("user", serde_json::to_string(user).unwrap()).unwrap();
}

pub fn get_current_user(session: &Session) -> Result<SessionUser, AuthError> {
    let msg = "Could not retrieve user from session";

    session.get::<String>("user")
        .map_err(|_| AuthError::AuthenticationError(String::from(msg)))
        .unwrap()
        .map_or(
          Err(AuthError::AuthenticationError(String::from(msg))),
          |user| serde_json::from_str(&user).or_else(|_| Err(AuthError::AuthenticationError(String::from(msg)))) 
        ) 
}

pub fn to_home() -> HttpResponse {
  HttpResponse::Found().header(LOCATION, "/me").finish()
}
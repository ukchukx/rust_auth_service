use actix_web::{error::BlockingError, web, HttpResponse};
use actix_session::Session;
use diesel::prelude::*;
use serde::Deserialize;

use crate::{
    models::{Confirmation, Pool, SessionUser, User}, 
    errors::AuthError, 
    schema::{
      confirmations::dsl::{id, confirmations},
      users::dsl::users
    },
    utils::{hash_password, is_signed_in, set_current_user}
};


#[derive(Debug, Deserialize)]
pub struct PasswordData {
    pub password: String,
}

pub async fn create_account(session: Session,
                            path_id: web::Path<String>,
                            data: web::Json<PasswordData>,
                            pool: web::Data<Pool>) -> Result<HttpResponse, AuthError> {
    if is_signed_in(&session) {
        return Ok(HttpResponse::BadRequest().finish());
    }

    let result = web::block(move || create_user(&path_id.into_inner(), &data.into_inner().password, &pool)).await;

    match result {
        Ok(user) => {
            set_current_user(&session, &user);

            Ok(HttpResponse::Created().json(&user))
        },
        Err(err) => match err {
            BlockingError::Error(auth_error) => Err(auth_error),
            BlockingError::Canceled => Err(AuthError::GenericError(String::from("Could not complete the process"))),
        },
    }
}


fn create_user(path_id: &str, password: &str, pool: &web::Data<Pool>) -> Result<SessionUser, AuthError> {
    let path_uuid = uuid::Uuid::parse_str(path_id)?;
    let conn = &pool.get().unwrap();

    confirmations
        .filter(id.eq(path_uuid))
        .load::<Confirmation>(conn)
        .map_err(|_db_error| AuthError::NotFound(String::from("Confirmation not found")))
        .and_then(|mut result| {
            if let Some(confirmation) = result.pop() {
                if confirmation.expires_at > chrono::Local::now().naive_local() { // confirmation has not expired
                    let password: String = hash_password(password)?;
                    let user: User = diesel::insert_into(users)
                                            .values(&User::from(confirmation.email, password))
                                            .get_result(conn)?;

                    return Ok(user.into());
                }
            }

            Err(AuthError::AuthenticationError(String::from("Invalid confirmation")))
        })
}
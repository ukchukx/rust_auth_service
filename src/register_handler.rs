use actix_web::{error::BlockingError, http, web, HttpResponse};
use actix_session::Session;
use diesel::prelude::*;
use serde::Deserialize;
use yarte::Template;

use crate::{
    email_service::send_confirmation_mail, 
    errors::AuthError, 
    models::{Confirmation, Pool},
    utils::is_signed_in
};


#[derive(Deserialize)]
pub struct RegisterData {
    pub email: String,
}

pub async fn send_confirmation(session: Session,
                              data: web::Json<RegisterData>,
                              pool: web::Data<Pool>) -> Result<HttpResponse, AuthError> {
    if (is_signed_in(&session)) {
      return Ok(HttpResponse::BadRequest().finish());
    }
            
    let result = web::block(move || create_confirmation(data.into_inner().email, &pool)).await;

    match result {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(err) => match err {
            BlockingError::Error(auth_error) => Err(auth_error),
            BlockingError::Canceled => Err(AuthError::GenericError(String::from("Could not complete the process"))),
        },
    }
}


fn create_confirmation(email: String, pool: &web::Data<Pool>) -> Result<(), AuthError> {
    let confirmation = insert_record(email, pool)?;

    send_confirmation_mail(&confirmation)
}

fn insert_record(email: String, pool: &web::Data<Pool>) -> Result<Confirmation, AuthError> {
    use crate::schema::confirmations::dsl::confirmations;

    let new_record : Confirmation = email.into();

    let inserted_record = diesel::insert_into(confirmations)
                                .values(&new_record)
                                .get_result(&pool.get().unwrap())?;

    Ok(inserted_record)
}
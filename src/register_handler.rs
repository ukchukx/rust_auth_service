use actix_web::{error::BlockingError, web, HttpResponse};
use actix_session::Session;
use diesel::prelude::*;
use serde::Deserialize;
use yarte::Template;

use crate::{
    email_service::send_confirmation_mail, 
    errors::AuthError, 
    models::{Confirmation, Pool},
    templates::Register,
    utils::{is_signed_in, to_home}
};


#[derive(Deserialize)]
pub struct RegisterData {
    pub email: String,
}

pub async fn send_confirmation(session: Session,
                              data: web::Json<RegisterData>,
                              pool: web::Data<Pool>) -> Result<HttpResponse, AuthError> {
    if is_signed_in(&session) {
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

pub async fn show_confirmation_form(session: Session) -> Result<HttpResponse, AuthError> {
    if is_signed_in(&session) {
        Ok(to_home())
    } else {
        let template = Register { sent: false, error: None };

        Ok(HttpResponse::Ok().content_type("text/html; charset=utf-8").body(template.call().unwrap()))
    }
}

pub async fn send_confirmation_for_browser(data: web::Form<RegisterData>,
                                          pool: web::Data<Pool>) -> Result<HttpResponse, AuthError> {
    let result = web::block(move || create_confirmation(data.into_inner().email, &pool)).await;
    let template = match result {
        Ok(_) => Register { sent: true, error: None },
        Err(err) => match err {
            BlockingError::Error(auth_error) => Register { sent: false, error: Some(auth_error.to_string()) },
            BlockingError::Canceled => {
                Register { sent: false, error: Some(String::from("Could not complete the process")) }
            }
        },
    };

    Ok(HttpResponse::Ok().content_type("text/html; charset=utf-8").body(template.call().unwrap()))
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
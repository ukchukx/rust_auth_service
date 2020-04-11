
use actix_session::Session;
use actix_web::{
    http::header::LOCATION,
    HttpRequest,
    HttpResponse,
    web
};
use diesel::prelude::*;
use serde::Deserialize;
use yarte::Template;

use crate::{
    models::{Pool, SessionUser, User},
    errors::AuthError,
    utils::{is_json_request, get_current_user, is_signed_in, set_current_user, to_home, verify}, 
    templates::{SignIn, Me}
};


pub async fn me(session: Session, req: HttpRequest) -> HttpResponse {
    let user_result = dbg!(get_current_user(&session));

    match is_json_request(&req) {
        true => {
            user_result.map_or(HttpResponse::Unauthorized().json("Unauthorized"), |user| HttpResponse::Ok().json(user))
        },
        false => {
            user_result.map_or(
                HttpResponse::MovedPermanently().header(LOCATION, "/signin").finish(), 
                |user| {
                    let t = Me { user };
            
                    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(t.call().unwrap())
                }
            )
        }
    }
}

pub async fn sign_out(session: Session, req: HttpRequest) -> HttpResponse {
    session.clear();
    
    match is_json_request(&req) {
        true => HttpResponse::NoContent().finish(),
        false => HttpResponse::MovedPermanently().header(LOCATION, "/signin").finish(),
    }
}

#[derive(Debug, Deserialize)]
pub struct AuthData {
    pub email: String,
    pub password: String,
}

pub async fn sign_in(data: web::Json<AuthData>, 
                  session: Session, 
                  req: HttpRequest,
                  pool: web::Data<Pool>) -> Result<HttpResponse, AuthError> {
    match is_signed_in(&session) {
        true => {
            let response = get_current_user(&session).map(|user| HttpResponse::Ok().json(user)).unwrap();

            Ok(response)
        },
        false => handle_sign_in(data.into_inner(), &session, &req, &pool)
    }
}

pub async fn show_sign_in_form(session: Session) -> Result<HttpResponse, AuthError> {
    match is_signed_in(&session) {
        true => Ok(to_home()),
        false => {
            let t = SignIn { error: None };

            Ok(HttpResponse::Ok().content_type("text/html; charset=utf-8").body(t.call().unwrap()))
        }
    }
}

pub async fn sign_in_for_browser(data: web::Form<AuthData>, 
                                session: Session, 
                                req: HttpRequest,
                                pool: web::Data<Pool>) -> Result<HttpResponse, AuthError> {
    handle_sign_in(data.into_inner(), &session, &req, &pool)
}

fn handle_sign_in(data: AuthData, 
                session: &Session, 
                req: &HttpRequest,
                pool: &web::Data<Pool>) -> Result<HttpResponse, AuthError> {
    let result = find_user(data, pool);
    let is_json = is_json_request(req);

    match result {
        Ok(user) => {
            set_current_user(&session, &user);

            if is_json {
                Ok(HttpResponse::Ok().json(user))
            } else {
                Ok(to_home())
            }
        },
        Err(err) => {
            if is_json {
                Ok(HttpResponse::Unauthorized().json(err.to_string()))
            } else {
                let t = SignIn { error: Some(err.to_string()) };
    
                Ok(HttpResponse::Ok().content_type("text/html; charset=utf-8").body(t.call().unwrap()))
            }
        },
    }
}

fn find_user(data: AuthData, pool: &web::Data<Pool>) -> Result<SessionUser, AuthError> {
    use crate::schema::users::dsl::{email, users};
    
    let mut items = users
        .filter(email.eq(&data.email))
        .load::<User>(&pool.get().unwrap())?;

    if let Some(user) = items.pop() {
        if let Ok(matching) = verify(&user.hash, &data.password) {
            if matching {
                return Ok(user.into());
            }
        }
    }

    Err(AuthError::NotFound(String::from("User not found")))
}

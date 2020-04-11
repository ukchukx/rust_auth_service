
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
    templates::Me
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

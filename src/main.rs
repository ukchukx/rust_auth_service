#[macro_use]
extern crate diesel;
extern crate serde_json;
extern crate lettre;
extern crate native_tls;

mod auth_handler;
mod email_service;
mod errors;
mod models;
mod password_handler;
mod register_handler;
mod schema;
mod templates;
mod utils;
mod vars;


#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    use actix_cors::Cors;
    use actix_files::Files;
    use actix_redis::RedisSession;
    use actix_web::{middleware, web, App, HttpServer};
    use diesel::{
        prelude::*, 
        r2d2::{self, ConnectionManager}
    };

    std::env::set_var("RUST_LOG", "actix_web=info,actix_server=info");
    env_logger::init();

    // create a database connection pool
    let manager = ConnectionManager::<PgConnection>::new(vars::database_url());
    let pool: models::Pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create a database connection pool.");

    // Start http server
    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            // enable logger
            .wrap(middleware::Logger::default())
            // Enable sessions
            .wrap(RedisSession::new("127.0.0.1:6379", &[0; 32]))
            .wrap(
                Cors::new()
                    .allowed_methods(vec!["GET", "POST", "DELETE", "OPTIONS"])
                    .max_age(3600)
                    .finish())
            .service(Files::new("/assets", "./templates/assets"))
            // Routes
            .service(
                web::scope("/")
                    .service(
                        web::resource("/register")
                            .route(web::get().to(register_handler::show_confirmation_form))
                            .route(web::post().to(register_handler::send_confirmation)),
                    )
                    .service(
                        web::resource("/register/{path_id}")
                            .route(web::get().to(password_handler::show_password_form))
                            .route(web::post().to(password_handler::create_account)),
                    )
                    .route("/register2/{path_id}", web::post().to(password_handler::create_account_for_browser))
                    .route("/register2", web::post().to(register_handler::send_confirmation_for_browser))
                    .route("/me", web::get().to(auth_handler::me))
                    .service(
                        web::resource("/signout")
                            .route(web::get().to(auth_handler::sign_out))
                            .route(web::delete().to(auth_handler::sign_out)),
                    )
                    .service(
                        web::resource("/signin")
                            .route(web::post().to(auth_handler::sign_in)),
                    ),
            )
    })
    .bind(format!("{}:{}", vars::domain(), vars::port()))?
    .run()
    .await
}

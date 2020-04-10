use diesel::{r2d2::ConnectionManager, PgConnection};

// type alias to reduce verbosity
pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
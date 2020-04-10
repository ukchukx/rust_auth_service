use diesel::{r2d2::ConnectionManager, PgConnection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::schema::*;

// type alias to reduce verbosity
pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "confirmations"]
pub struct Confirmation {
    pub id: Uuid,
    pub email: String,
    pub expires_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "users"]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub hash: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionUser {
    pub id: Uuid,
    pub email: String,
}

// any type that implements Into<String> can be used to create a Confirmation
impl<T> From<T> for Confirmation where
T: Into<String> {
     fn from(email: T) -> Self {
        Confirmation {
            id: Uuid::new_v4(),
            email: email.into(),
            expires_at: chrono::Local::now().naive_local() + chrono::Duration::hours(24),
        }
    }
}

impl From<User> for SessionUser {
    fn from(User { email, id, .. }: User) -> Self {
        SessionUser { email, id }
    }
}

impl User {
    pub fn from<S: Into<String>, T: Into<String>>(email: S, pwd: T) -> Self {
        User {
            id: Uuid::new_v4(),
            email: email.into(),
            hash: pwd.into(),
            created_at: chrono::Local::now().naive_local(),
        }
    }
}
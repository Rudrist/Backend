use crate::db_lib::database;
use crate::db_lib::schema;
use ::diesel::ExpressionMethods;
use pbkdf2::password_hash::PasswordHasher;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::serde::json::{json, Value};
use rocket_db_pools::diesel::prelude::RunQueryDsl;
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
// use crate::types::ResponseData;
// The signup info of the user. Simple constraints are checked in the front end (html).
#[derive(Serialize, Deserialize)]
pub struct SignupInfo<'r> {
    name: &'r str,
    password: &'r str,
    email: &'r str,
    user_type: i32,
}

// TODO, signup is available only when not logged in
// if signup sucessfully, redirect to login page. (It won't log in automatically)
// Otherwise, return Status::BadRequest and a string indicating the error. (It is not fancy at all :< )
#[post("/api/auth/user", data = "<signup_info>")]
pub async fn signup(
    signup_info: Json<SignupInfo<'_>>,
    mut db_conn: Connection<database::PgDb>,
) -> (Status, Value) {
    // hash the password
    let salt = pbkdf2::password_hash::SaltString::generate(&mut rand_core::OsRng);
    let password_hash = pbkdf2::Pbkdf2.hash_password(signup_info.password.as_bytes(), &salt);
    let hashed_password = if let Ok(_password) = password_hash {
        _password.to_string()
    } else {
        return (
            Status::BadRequest,
            json!({"status":"error", "message": "The password is invalid."}),
        );
    };

    // inser the signup user data into the database
    let signup_user_id = rocket_db_pools::diesel::insert_into(schema::accounts::table)
        .values((
            schema::accounts::username.eq(signup_info.name.to_string()),
            schema::accounts::email.eq(signup_info.email.to_string()),
            schema::accounts::password.eq(&hashed_password),
            schema::accounts::account_type.eq(signup_info.user_type),
        ))
        .execute(&mut db_conn)
        .await;

    // if the user data is inserted successfully, redirect to login page
    match signup_user_id {
        Ok(_) => {
            return (Status::Ok, json!({"status": "successful"}));
        }
        Err(diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _)) => {
            return (
                Status::BadRequest,
                json!({"status":"error", "message": "Account already exist."}),
            );
        }
        Err(err) => {
            println!("{:?}", err);
            return (
                Status::BadRequest,
                json!({"message": "Server Error."}),
            );
        }
    }
}

use ::diesel::ExpressionMethods;
use diesel::query_dsl::methods::{FilterDsl, SelectDsl};
use pbkdf2::password_hash::PasswordHasher;
use rocket::http::{CookieJar, Status};
use rocket::response::Redirect;
use rocket_db_pools::diesel::prelude::RunQueryDsl;
use rocket_db_pools::{diesel, Connection};
use rocket::serde::json::Json;
use rocket::serde::json::{json, Value};
use serde::{Deserialize, Serialize};

use crate::auth::validation::UserAuth;
use crate::db_lib::schema::{accounts, sessions};
use crate::db_lib::session::SessionToken;
use crate::db_lib::{database, USER_COOKIE_NAME};

// return the user_id according to the session token from the client(cookie)
pub async fn get_logged_in_user_id(
    cookies: &CookieJar<'_>,
    mut db_conn: &mut Connection<database::PgDb>,
) -> Option<i32> {
    // get the session token from the client(cookie)
    let fetch_cookie = cookies
        .get_private(USER_COOKIE_NAME)
        .and_then(|cookie| cookie.value().parse::<String>().ok());

    let session_token = if let Some(cookie) = fetch_cookie {
        SessionToken::to_token(cookie)
    } else {
        return None;
    };

    // get the user id corresponding to the session token from the database
    let fetch_user_id = sessions::table
        .select(sessions::user_id)
        .filter(sessions::session_token.eq(session_token.into_database_value()))
        .first::<i32>(&mut db_conn)
        .await;

    if let Ok(user_id) = fetch_user_id {
        return Some(user_id);
    } else {
        return None;
    };
}

// update the (hashed)password on the database
pub async fn set_new_password(
    user_id: i32,
    new_password: &str,
    mut db_conn: &mut Connection<database::PgDb>,
) -> Result<Redirect, (Status, &'static str)> {
    // hash the new password
    let salt = pbkdf2::password_hash::SaltString::generate(&mut rand_core::OsRng);
    let password_hash = pbkdf2::Pbkdf2.hash_password(new_password.as_bytes(), &salt);
    let new_hashed_password = if let Ok(_password) = password_hash {
        _password.to_string()
    } else {
        return Err((Status::BadRequest, "The new password is invalid."));
    };

    // update the database
    let update_password =
        rocket_db_pools::diesel::update(accounts::table.filter(accounts::id.eq(user_id)))
            .set(accounts::password.eq(new_hashed_password))
            .execute(&mut db_conn)
            .await;

    match update_password {
        Ok(_) => return Ok(Redirect::to(uri!("/index"))),
        Err(_) => return Err((Status::BadRequest, "Update password fails.")),
    }
}


#[derive(Serialize, Deserialize)]
pub(crate) struct ResetPasswordInfo<'r> {
    password: &'r str,
}

// if signup sucessfully, redirect to login page. (It won't log in automatically)
// Otherwise, return Status::BadRequest and a string indicating the error. (It is not fancy at all :< )
#[post("/reset_password", data = "<reset_password_info>")]
pub async fn reset_password(
    reset_password_info: Form<Strict<ResetPasswordInfo<'_>>>,
    mut db_conn: Connection<database::PgDb>,
    _user_auth: UserAuth,
) -> (Status, Value) {
    // ensure the user is logged in
    let user_id = _user_auth.user_id;

    let reset_result = set_new_password(user_id, reset_password_info.password, &mut db_conn).await;
    match reset_result {
        Ok(_) => {
            return (Status::Ok, json!({"status":"successful"}));
        }
        Err(_) => {
            return (
                Status::ServiceUnavailable,
                json!({"message": "Fail to reset password."}),
            );
        }
    }
}

// remove the session token from both the server(database) and the client(cookie)
#[get("/logout")]
    mut db_conn: Connection<database::PgDb>,
    cookies: &CookieJar<'_>,
    _user_auth: UserAuth,
) -> (Status, Value) {
    // Ensure the user is logged in
    let user_id = _user_auth.user_id;

    // remove session token from server(database) and client(cookie)
    cookies.remove_private(USER_COOKIE_NAME);
    let logout_result = diesel::delete(sessions::table.filter(sessions::user_id.eq(user_id))).execute(&mut db_conn).await;

    match logout_result {
        Ok(_) => { 
            return (Status::Ok, json!({"status":"successful"}));
        }
        Err(_) => {
            return (
                Status::BadRequest,
                json!({"message": "Fail to remove session token in the database."}),
            );
        }
    }
}

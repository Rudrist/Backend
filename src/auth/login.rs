use crate::db_lib::schema::accounts;
use crate::db_lib::session::new_session;
use crate::db_lib::USER_COOKIE_NAME;
use crate::db_lib::{database, RAND};
use ::diesel::ExpressionMethods;
use diesel::query_dsl::methods::{FilterDsl, SelectDsl};
use pbkdf2::password_hash::PasswordHash;
use pbkdf2::{password_hash::PasswordVerifier, Pbkdf2};
use rocket::http::{Cookie, CookieJar, Status};
use rocket::serde::json::Json;
use rocket::serde::json::{json, Value};
use rocket::State;
use rocket_db_pools::diesel::prelude::RunQueryDsl;
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct LoginInfo<'r> {
    name: &'r str,
    password: &'r str,
}

// If login successfully, a session token will be saved in both server(database) and the client(cookie), finally redirect to index page
// Otherwise, Status::Badrequest is returned (not fancy at all)
#[post("/api/auth/login", data = "<login_info>")]
pub async fn login(
    login_info: Json<LoginInfo<'_>>,
    mut db_conn: Connection<database::PgDb>,
    cookies: &CookieJar<'_>,
    random: &State<RAND>,
) -> (Status, Value) {
    // query the id and (hashed)password in the database according to the username
    let login_result = accounts::table
        .select((accounts::id, accounts::password))
        .filter(accounts::username.eq(login_info.name.to_string()))
        .first::<(i32, String)>(&mut db_conn)
        .await;

    // If query fails, return badquest
    let (user_id, hashed_password) = if let Ok(login_result_ok) = login_result {
        login_result_ok
    } else {
        return (
            Status::BadRequest,
            json!({"status":"error", "message":"Login fails. Probably wrong username or password."}),
        );
    };

    // If (hashed)password doesn't match, return badrequest
    if let Err(_err) = Pbkdf2.verify_password(
        login_info.password.as_bytes(),
        &PasswordHash::new(&hashed_password).unwrap(),
    ) {
        return (
            Status::BadRequest,
            json!({"status":"error", "message":"Wrong password."}),
        );
    }

    // Generate a session key. Save it in both the server(database) and the client(cookie).
    let token = new_session(random.random.clone(), user_id, &mut db_conn).await;
    match token {
        Ok(token) => {
            let cookie_value = token.into_cookie_value();
            cookies.add_private(Cookie::build((USER_COOKIE_NAME, cookie_value.clone()))); // default expire time: one week from now
            let fetch_account_type = accounts::table
            .select(accounts::account_type)
            .filter(accounts::id.eq(user_id))
            .first::<Option<i32>>(&mut db_conn)
            .await
            .unwrap();
            if let Some(account_type) = fetch_account_type {
                cookies.add(Cookie::build(("account_type", account_type.to_string())));
            } else {
                cookies.add(Cookie::build(("account_type", 1.to_string())));
            }

            return (Status::Ok, json!({"status":"successful"}));
        }
        Err(session_err) => {
            return (
                Status::ServiceUnavailable,
                json!({"status":"error", "message": session_err}),
            );
        }
    }
}

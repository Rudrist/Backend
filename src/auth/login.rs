use std::env;

use diesel::query_dsl::methods::{FilterDsl, SelectDsl};
use dotenv::dotenv;
use rand_core::{OsRng, RngCore};
use rocket::State;
use rocket::form::{Form, Strict};
use rocket::response::Redirect;
use rocket::http::{Cookie, CookieJar, Status};
use rocket_db_pools::Connection;
use rocket_db_pools::diesel::prelude::RunQueryDsl;
use ::diesel::ExpressionMethods;
use pbkdf2::password_hash::PasswordHash;
use pbkdf2::{password_hash::PasswordVerifier, Pbkdf2,};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use chrono::{Duration, Utc};
use urlencoding::encode;

use crate::db_lib::schema::users;
use crate::db_lib::session::new_session;
use crate::auth::user_center::get_logged_in_user_id;
use crate::db_lib::USER_COOKIE_NAME;
use crate::db_lib::{database, RAND};


#[derive(FromForm)]
pub(crate) struct LoginInfo<'r> {
    user_name: &'r str,
    user_password: &'r str
}

// If login successfully, a session token will be saved in both server(database) and the client(cookie), finally redirect to index page
// Otherwise, Status::Badrequest is returned (not fancy at all)
#[post("/api/auth/login", data = "<login_info>")]
pub(crate) async fn login(
    login_info: Form<Strict<LoginInfo<'_>>>,
    mut accounts_db_coon: Connection<database::AccountsDb>, 
    cookies: &CookieJar<'_>,
    random: &State<RAND>
) -> Result<(Status, String), (Status, &'static str)> {
    
    // query the id and (hashed)password in the database according to the username
    let login_result = users::table
        .select((users::id, users::password))
        .filter(users::username.eq(login_info.user_name.to_string()))
        .first::<(i32, String)>(&mut accounts_db_coon).await;

    // If query fails, return badquest
    let (user_id, hashed_password) = if let Ok(login_result_ok) = login_result {
        login_result_ok
    } else {
        return Err((Status::BadRequest, "Login fails. Probably wrong username or password."));
    };

    // If (hashed)password doesn't match, return badrequest
    if let Err(_err) = Pbkdf2.verify_password(login_info.user_password.as_bytes(), &PasswordHash::new(&hashed_password).unwrap()) {
        return Err((Status::BadRequest, "Wrong password."));
    }

    // Generate a session key. Save it in both the server(database) and the client(cookie).
    let token = new_session(random.random.clone(), user_id, &mut accounts_db_coon).await;
    match token {
        Ok(token) => {
            let cookie_value = token.into_cookie_value();
            cookies.add_private(Cookie::build((USER_COOKIE_NAME, cookie_value.clone()))); // default expire time: one week from now
            
            return Ok((Status::Ok, cookie_value));
        },
        Err(session_err) => {
            return Err(session_err);
        }
    }
}




use diesel::prelude::*;

use rocket::State;
use rocket::form::{Form, Strict};
use rocket::http::{Cookie, CookieJar, Status};
use ::diesel::ExpressionMethods;
use pbkdf2::password_hash::PasswordHash;
use pbkdf2::{password_hash::PasswordVerifier, Pbkdf2,};

use crate::db_lib::USER_COOKIE_NAME;
use crate::db_lib::{database::establish_connection, RAND};
use crate::db_lib::models::*;

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
    cookies: &CookieJar<'_>,
    random: &State<RAND>
) -> Result<(Status, String), (Status, &'static str)> {
    
    use crate::db_lib::schema::accounts::dsl::*;
    let connection = &mut establish_connection();
    // query the id and (hashed)password in the database according to the username
    let login_result:Vec<(i32, String)> = accounts
        .filter(username.eq(login_info.user_name.to_string()))
        .select((id, password))
        .load(connection)
        .expect("Error loading.");

    // If query fails, return badquest
    let user_id: i32;
    let hashed_password;
    if login_result.len() != 0 {
        (user_id, hashed_password) = login_result[0].clone();
    } else {
        return Err((Status::BadRequest, "Login fails. Probably wrong username or password."));
    };

    // If (hashed)password doesn't match, return badrequest
    if let Err(_err) = Pbkdf2.verify_password(login_info.user_password.as_bytes(), &PasswordHash::new(&hashed_password).unwrap()) {
        return Err((Status::BadRequest, "Wrong password."));
    }

    // Generate a session key. Save it in both the server(database) and the client(cookie).
    let token = "asd";
    return Ok((Status::Ok, token.to_string()));
}




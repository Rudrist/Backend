use rocket::form::{Form, Strict};
use rocket::http::{CookieJar, Status};
use rocket::response::Redirect;
use rocket_db_pools::{diesel, Connection};
use rocket_db_pools::diesel::prelude::RunQueryDsl;
use ::diesel::ExpressionMethods;
use diesel::query_dsl::methods::{FilterDsl, SelectDsl};
use pbkdf2::password_hash::{PasswordHash, PasswordHasher};
use pbkdf2::{password_hash::PasswordVerifier, Pbkdf2,};

use crate::db_lib::{database, USER_COOKIE_NAME};
use crate::db_lib::schema::{sessions, accounts};
use crate::db_lib::session::SessionToken;

// return the user_id according to the session token from the client(cookie)
pub(crate) async fn get_logged_in_user_id(
    cookies: &CookieJar<'_>,
    mut accounts_db_coon: &mut Connection<database::AccountsDb>
) -> Option<i32> {
    // get the session token from the client(cookie)
    let fetch_cookie = cookies.get_private(USER_COOKIE_NAME)
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
        .first::<i32>(&mut accounts_db_coon).await;
    
    if let Ok(user_id) = fetch_user_id {
        return Some(user_id);
    } else {
        return None;
    };
}

// update the (hashed)password on the database
pub(crate) async fn set_new_password(
    user_id: i32, 
    new_password: &str, 
    mut accounts_db_coon: &mut Connection<database::AccountsDb>
) -> Result<Redirect, (Status, &'static str)> {
    // hash the new password
    let salt = pbkdf2::password_hash::SaltString::generate(&mut rand_core::OsRng);
    let password_hash = pbkdf2::Pbkdf2.hash_password(new_password.as_bytes(), &salt);
    let new_hashed_password = if let Ok(_password) = password_hash {
        _password.to_string()
    } else {
        return Err((Status::BadRequest, "The new password is invalid."))
    };
    
    // update the database
    let update_password = rocket_db_pools::diesel::update(accounts::table.filter(accounts::id.eq(user_id)))
        .set(accounts::password.eq(new_hashed_password))
        .execute(&mut accounts_db_coon).await;

    match update_password {
        Ok(_) => return Ok(Redirect::to(uri!("/index"))),
        Err(_) => return Err((Status::BadRequest, "Update password fails."))
    }
}

#[derive(FromForm)]
pub(crate) struct ResetPasswordInfo<'r> {
    current_password: &'r str,
    new_password: &'r str
}

// if signup sucessfully, redirect to login page. (It won't log in automatically)
// Otherwise, return Status::BadRequest and a string indicating the error. (It is not fancy at all :< )
#[post("/reset_password", data = "<reset_password_info>")]
pub(crate) async fn reset_password(
    reset_password_info: Form<Strict<ResetPasswordInfo<'_>>>, 
    mut accounts_db_coon: Connection<database::AccountsDb>,
    cookies: &CookieJar<'_>
) -> Result<Redirect, (Status, &'static str)> {

    // ensure the user is logged in
    let user_id = if let Some(user_id) = get_logged_in_user_id(cookies, &mut accounts_db_coon).await {
        user_id
    } else {
        return Err((Status::BadRequest, "Cannot fetch user id based on session token cookie or cookie crushed."));
    };

    // fetch the (hashed)user password from the database
    let fetch_user_password = accounts::table
        .select(accounts::password)
        .filter(accounts::id.eq(user_id))
        .first::<String>(&mut accounts_db_coon).await;

    let current_hashed_password = if let Ok(password) = fetch_user_password {
        password
    } else {
        return Err((Status::BadRequest, "Fail to fetch password from database"));
    };

    // If (hashed)current_password doesn't match, return badrequest
    if let Err(_err) = Pbkdf2.verify_password(
        reset_password_info.current_password.as_bytes(), 
        &PasswordHash::new(&current_hashed_password).unwrap()) 
    {
        return Err((Status::BadRequest, "Current password wrong."));
    }

    set_new_password(user_id, reset_password_info.new_password, &mut accounts_db_coon).await
}

// remove the session token from both the server(database) and the client(cookie)
#[get("/logout")]
pub(crate) async fn logout(
    mut accounts_db_coon: Connection<database::AccountsDb>, 
    cookies: &CookieJar<'_>
) -> Result<Redirect, (Status, &'static str)> {
    // Ensure the user is logged in
    let user_id = if let Some(user_id) = get_logged_in_user_id(&cookies, &mut accounts_db_coon).await {
        user_id
    } else {
        return Err((Status::BadRequest, "Not logged in."));
    };
    
    // remove session token from server(database) and client(cookie)
    cookies.remove_private(USER_COOKIE_NAME);
    return match diesel::delete(sessions::table.filter(sessions::user_id.eq(user_id))).execute(&mut accounts_db_coon).await {
        Ok(_) => Ok(Redirect::to(uri!("/login"))),
        Err(_) => Err((Status::BadRequest, "Fail to remove session token in the database"))
    };   
}
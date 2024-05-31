use std::env;

use ::diesel::ExpressionMethods;
use chrono::{Duration, Utc};
use diesel::query_dsl::methods::{FilterDsl, SelectDsl};

use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use rand_core::{OsRng, RngCore};
use rocket::http::{CookieJar, Status};
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::auth::user_center::get_logged_in_user_id;
use crate::db_lib::schema::accounts;
use crate::db_lib::database;


#[derive(Serialize, Deserialize)]
pub(crate) struct ForgetPasswordInfo<'r> {
    name: &'r str,
}

/* 
#[post("/api/auth/forget", data = "<forget_password_info>")]
pub async fn forget_password(
    forget_password_info: Form<Strict<ForgetPasswordInfo<'_>>>,
    mut db_conn: Connection<database::PgDb>,
    cookies: &CookieJar<'_>,
) -> (Status, Value) {
    if let Some(_) = get_logged_in_user_id(cookies, &mut db_conn).await {
        return Err((Status::BadRequest, "Already Logged in."));
    }

    let username = forget_password_info.name.to_string();

    // Query for user email
    let fetch_user_email = accounts::table
        .select(accounts::email)
        .filter(accounts::username.eq(username))
        .first::<String>(&mut db_conn)
        .await;

    let user_email = match fetch_user_email {
        Ok(email) => email,
        Err(err) => {
            return (
                Status::BadRequest,
                json!({"status":"error", "message": format!("User email not found: {}", err)}),
            );
        }
    };

    let reset_token = OsRng.next_u64();
    let send_email =
        send_reset_password_email(forget_password_info.user_name, &user_email, &reset_token).await;

    match send_email {
        Ok(_) => {
            return (Status::Ok, json!({"status":"successful"}));
        }Err(_) =>{
            return (
                Status::ServiceUnavailable,
                json!({"status":"error", "message":"Fail to send the email."}),
            );
        }
    }
}
*/

pub async fn send_reset_password_email(
    user_name: &str,
    user_email: &str,
    reset_token: &u64,
) -> Result<lettre::transport::smtp::response::Response, lettre::transport::smtp::Error> {
    let smtp_key: &str = "pA6ZPCjEVv7U0Grz";
    let from_email: &str = "testgdscmail@gmail.com";
    let to_email: &str = &user_email;
    let host: &str = "smtp-relay.sendinblue.com";
    let expiration_time = Utc::now() + Duration::minutes(5);
    let reset_link = format!(
        "{}/api/auth/forget/{}/{}/{}",
        env::var("DOMAIN").unwrap_or_default(),
        user_name,
        reset_token,
        encode(&expiration_time.to_rfc3339())
    );

    let email: Message = Message::builder()
        .from(from_email.parse().unwrap())
        .to(to_email.parse().unwrap())
        .subject("Reset your password")
        .body(format!(
            "Please click the following link to reset your password:\nLink:{}\nThe link will expired in 5 minutes.",
            reset_link))
        .unwrap();

    let creds: Credentials = Credentials::new(from_email.to_string(), smtp_key.to_string());

    // Open a remote connection to gmail
    let mailer: SmtpTransport = SmtpTransport::relay(&host)
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    return mailer.send(&email);
}

// TO DO post(forget)
/*
#[derive(FromForm)]
pub struct ResetPasswordInfo<'r> {
    user_password: &'r str,
    confirm_password: &'r str
}

#[post("/api/auth/forget/<username>/<resettoken>/<expiration_timestamp>", data = "<reset_info>")]
pub async fn reset_password(
    reset_info: Form<Strict<SignupInfo<'_>>>,
    mut db_conn: Connection<database::AccountsDb>
) -> Result<Status, (Status, &'static str)> {

    // confirm the password
    if reset_info.user_password != reset_info.confirm_password {
        return Err((Status::BadRequest, "The password doesn't match."));
    }

    // hash the password
    let salt = pbkdf2::password_hash::SaltString::generate(&mut rand_core::OsRng);
    let password_hash = pbkdf2::Pbkdf2.hash_password(signup_info.user_password.as_bytes(), &salt);
    let hashed_password = if let Ok(_password) = password_hash {
        _password.to_string()
    } else {
        return Err((Status::BadRequest, "The password is invalid."))
    };

    // update the user's password in the database
    let update_result = diesel::update(users::table.filter(users::username.eq(&reset_info.user_name)))
        .set(users::password.eq(&hashed_password))
        .execute(&mut accounts_db_coon)
        .await;

    match update_result {
        Ok(_) => {
            return Ok(Status::Ok);
        }
        Err(_) => {
            return Err((Status::InternalServerError, "Failed to update the password."));
        }
    }

}
*/

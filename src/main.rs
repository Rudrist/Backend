#[macro_use] extern crate rocket;

use std::path::{Path, PathBuf};
use rand_core::{SeedableRng, RngCore};
use rocket::http::{CookieJar, Status};
use rocket_db_pools::{Connection, Database};
use rocket::fs::NamedFile;
use rocket::response::content::RawHtml;
use std::sync::{Arc, Mutex};
use rocket::fs::FileServer;


mod db_lib;
use db_lib::{database, RAND};
mod auth;
use auth::{login, signup, user_center, forget};


#[get("/")]
fn index() -> RawHtml<&'static str> {
    return RawHtml(include_str!("../static/index.html"));
}
#[get("/api/auth/register")]
async fn signup_page(
    mut accounts_db_coon: Connection<database::AccountsDb>, 
    cookies: &CookieJar<'_>
) -> Result<RawHtml<&'static str>, (Status, &'static str)> {

    if let Some(_) = user_center::get_logged_in_user_id(cookies, &mut accounts_db_coon).await {
        return Err((Status::BadRequest, "Already logged in."));
    }
    return Ok(RawHtml(include_str!("../static/signup.html")));
}
#[get("/api/auth/login")]
async fn login_page(
    mut accounts_db_coon: Connection<database::AccountsDb>, 
    cookies: &CookieJar<'_>
) -> Result<RawHtml<&'static str>, (Status, &'static str)> {

    if let Some(_) = user_center::get_logged_in_user_id(cookies, &mut accounts_db_coon).await {
        return Err((Status::BadRequest, "Already logged in."));
    }
    return Ok(RawHtml(include_str!("../static/login.html")));
}

#[get("/user_center")]
async fn user_center_page(
    mut accounts_db_coon: Connection<database::AccountsDb>, 
    cookies: &CookieJar<'_>
) -> Result<RawHtml<&'static str>, (Status, &'static str)> {
    
    if let None = user_center::get_logged_in_user_id(cookies, &mut accounts_db_coon).await {
        return Err((Status::BadRequest, "Not yet logged in."));
    }
    return Ok(RawHtml(include_str!("../static/user_center.html")));
}

#[get("/api/auth/forget")]
async fn forget_page(
    mut accounts_db_coon: Connection<database::AccountsDb>, 
    cookies: &CookieJar<'_>
) -> Result<RawHtml<&'static str>, (Status, &'static str)> {
    if let Some(_) = user_center::get_logged_in_user_id(cookies, &mut accounts_db_coon).await {
        return Err((Status::BadRequest, "Already logged in."));
    }
    return Ok(RawHtml(include_str!("../static/forget.html")));
}


//TO DO get(reset_page)
/*
#[get("/api/auth/forget/<username>/<resettoken>/<expiration_timestamp>")]
async fn reset_page(
    username: String,
    resettoken: String,
    expiration_timestamp: String,
    mut accounts_db_coon: Connection<database::AccountsDb>, 
    cookies: &CookieJar<'_>,
) -> Result<(Status, &'static str), (Status, &'static str)> {
    let expiration_time = expiration_timestamp.parse::<DateTime<Utc>>();
    match expiration_time {
        Ok(expiration_time) => {
            let current_time = Utc::now();
            let duration = expiration_time.signed_duration_since(current_time);
            if duration.num_minutes() > 5 {
                return Err((Status::BadRequest, "Link has expired."));
            }
            return Ok((Status::Ok, "Successful"));
        },
        Err(_) => {
            return Err((Status::BadRequest, "Invalid expiration timestamp format."));
        }
    }
}
*/

#[get("/<file..>")]
async fn files(file: PathBuf) -> Option<NamedFile> {
    println!("{}", Path::new("./static/").join(&file).to_str().unwrap());
    NamedFile::open(Path::new("./static/").join(file)).await.ok()
}



#[rocket::main]
async fn main() {

    rocket::build()
        .attach(database::AccountsDb::init())
        .manage(RAND {random: Arc::new(Mutex::new(rand_chacha::ChaCha8Rng::seed_from_u64(rand_core::OsRng.next_u64())))})
        .mount("/", FileServer::from("static"))
        .mount("/", routes![files])
        .mount("/", routes![index])
        .mount("/", routes![signup_page, signup::signup])
        .mount("/", routes![login_page, login::login])
        .mount("/", routes![user_center_page, user_center::reset_password, user_center::logout])
        .mount("/", routes![forget_page, forget::forget_password])
        .launch()
        .await
        .expect("Failed to launch rocket");

}
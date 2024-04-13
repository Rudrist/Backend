#[macro_use]
extern crate rocket;

use rand_core::{RngCore, SeedableRng};
use rocket::fs::NamedFile;
use rocket::http::{CookieJar, Status};
use rocket::response::content::RawHtml;
use rocket_db_pools::{Connection, Database};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

mod db_lib;
use db_lib::{database, RAND};
mod auth;
use auth::{login, signup, user_center};
mod ledger;

#[get("/")]
fn index() -> RawHtml<&'static str> {
    return RawHtml(include_str!("../static/index.html"));
}
#[get("/signup")]
async fn signup_page(
    mut accounts_db_coon: Connection<database::AccountsDb>,
    cookies: &CookieJar<'_>,
) -> Result<RawHtml<&'static str>, (Status, &'static str)> {
    if let Some(_) = user_center::get_logged_in_user_id(cookies, &mut accounts_db_coon).await {
        return Err((Status::BadRequest, "Already logged in."));
    }
    return Ok(RawHtml(include_str!("../static/signup.html")));
}
#[get("/login")]
async fn login_page(
    mut accounts_db_coon: Connection<database::AccountsDb>,
    cookies: &CookieJar<'_>,
) -> Result<RawHtml<&'static str>, (Status, &'static str)> {
    if let Some(_) = user_center::get_logged_in_user_id(cookies, &mut accounts_db_coon).await {
        return Err((Status::BadRequest, "Already logged in."));
    }
    return Ok(RawHtml(include_str!("../static/login.html")));
}

#[get("/user_center")]
async fn user_center_page(
    mut accounts_db_coon: Connection<database::AccountsDb>,
    cookies: &CookieJar<'_>,
) -> Result<RawHtml<&'static str>, (Status, &'static str)> {
    if let None = user_center::get_logged_in_user_id(cookies, &mut accounts_db_coon).await {
        return Err((Status::BadRequest, "Not yet logged in."));
    }
    return Ok(RawHtml(include_str!("../static/user_center.html")));
}

#[get("/<file..>")]
async fn files(file: PathBuf) -> Option<NamedFile> {
    println!("{}", Path::new("./static/").join(&file).to_str().unwrap());
    NamedFile::open(Path::new("./static/").join(file))
        .await
        .ok()
}

#[rocket::main]
async fn main() {
    rocket::build()
        .attach(database::AccountsDb::init())
        .manage(RAND {
            random: Arc::new(Mutex::new(rand_chacha::ChaCha8Rng::seed_from_u64(
                rand_core::OsRng.next_u64(),
            ))),
        })
        .mount("/", routes![files])
        .mount("/", routes![index])
        .mount("/index", routes![index])
        .mount("/", routes![signup_page, signup::signup])
        .mount("/", routes![login_page, login::login])
        .mount(
            "/",
            routes![
                user_center_page,
                user_center::reset_password,
                user_center::logout,
                login::forget_password
            ],
        )
        .launch()
        .await
        .expect("Failed to launch rocket");
}

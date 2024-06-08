#[macro_use]
extern crate rocket;

use rand_core::{RngCore, SeedableRng};
use rocket::fs::FileServer;
use rocket::fs::NamedFile;
use rocket::http::{CookieJar, Status};
use rocket::response::content::RawHtml;
use rocket::serde::json::{json, Value};
use rocket_db_pools::{Connection, Database};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

mod db_lib;
use db_lib::{database, RAND};
mod auth;
use auth::{login, signup, user_center};
mod portfolio;
use rocket::http::Method;
use rocket_cors::{AllowedOrigins, CorsOptions};
mod order;
mod risk;

use crate::db_lib::schema::accounts;
use ::diesel::ExpressionMethods;
use diesel::query_dsl::methods::{FilterDsl, SelectDsl};
use rocket_db_pools::diesel::prelude::RunQueryDsl;

#[get("/")]
fn index() -> RawHtml<&'static str> {
    return RawHtml(include_str!("../static/index.html"));
}
#[get("/api/auth/user")]
async fn signup_page(
    mut db_conn: Connection<database::PgDb>,
    cookies: &CookieJar<'_>,
) -> (Status, Value) {
    if let Some(_) = user_center::get_logged_in_user_id(cookies, &mut db_conn).await {
        if let Some(user_id) = user_center::get_logged_in_user_id(cookies, &mut db_conn).await {
            let account_type = accounts::table
                .select(accounts::account_type)
                .filter(accounts::id.eq(user_id))
                .first::<Option<i32>>(&mut db_conn)
                .await
                .unwrap()
                .unwrap();

            return (
                Status::Ok,
                json!({"status":"Login", "account_type": account_type}),
            );
        }
    }
    return (Status::Ok, json!({"status":"Logout"}));
}
#[get("/api/auth/login")]
async fn login_page(
    mut db_conn: Connection<database::PgDb>,
    cookies: &CookieJar<'_>,
) -> Result<RawHtml<&'static str>, (Status, &'static str)> {
    if let Some(_) = user_center::get_logged_in_user_id(cookies, &mut db_conn).await {
        return Err((Status::BadRequest, "Already logged in."));
    }
    return Ok(RawHtml(include_str!("../static/login.html")));
}

#[get("/user_center")]
async fn user_center_page(
    mut db_conn: Connection<database::PgDb>,
    cookies: &CookieJar<'_>,
) -> Result<RawHtml<&'static str>, (Status, &'static str)> {
    if let None = user_center::get_logged_in_user_id(cookies, &mut db_conn).await {
        return Err((Status::BadRequest, "Not yet logged in."));
    }
    return Ok(RawHtml(include_str!("../static/user_center.html")));
}

// #[get("/api/auth/forget")]
// async fn forget_page(
//     mut db_conn: Connection<database::PgDb>,
//     cookies: &CookieJar<'_>,
// ) -> Result<RawHtml<&'static str>, (Status, &'static str)> {
//     if let Some(_) = user_center::get_logged_in_user_id(cookies, &mut db_conn).await {
//         return Err((Status::BadRequest, "Already logged in."));
//     }
//     return Ok(RawHtml(include_str!("../static/forget.html")));
// }

/*
#[get("/api/portfolio")]
async fn portfolio_page(
    mut accounts_db_coon: Connection<database::PgDb>,
    cookies: &CookieJar<'_>,
) -> Result<RawHtml<&'static str>, (Status, &'static str)> {
    if let None = user_center::get_logged_in_user_id(cookies, &mut accounts_db_coon).await {
        return Err((Status::BadRequest, "Not yet logged in."));
    }
    return Ok(RawHtml(include_str!("../static/portfolio.html")));
}
*/

//TO DO get(reset_page)
/*
#[get("/api/auth/forget/<username>/<resettoken>/<expiration_timestamp>")]
async fn reset_page(
    username: String,
    resettoken: String,
    expiration_timestamp: String,
    mut db_conn: Connection<database::PgDb>,
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
    NamedFile::open(Path::new("./static/").join(file))
        .await
        .ok()
}

#[rocket::main]
async fn main() {
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![
                Method::Get,
                Method::Post,
                Method::Patch,
                Method::Delete,
                Method::Put,
            ]
            .into_iter()
            .map(From::from)
            .collect(),
        )
        .allow_credentials(true);
    rocket::build()
        .attach(database::PgDb::init())
        .attach(cors.to_cors().unwrap())
        .manage(RAND {
            random: Arc::new(Mutex::new(rand_chacha::ChaCha8Rng::seed_from_u64(
                rand_core::OsRng.next_u64(),
            ))),
        })
        .mount("/", FileServer::from("static"))
        .mount("/", routes![files])
        .mount("/", routes![index])
        .mount("/", routes![signup_page, signup::signup])
        .mount("/", routes![login_page, login::login])
        .mount(
            "/",
            routes![
                user_center_page,
                user_center::reset_password,
                user_center::logout
            ],
        )
        // .mount("/", routes![forget_page, forget::forget_password])
        .mount(
            "/",
            routes![
                portfolio::create_portfolio::add_portfolio,
                portfolio::remove_portfolio::remove_portfolio,
                portfolio::get_portfolio::get_portfolio_names,
                portfolio::change_portfolio::change_portfolio
            ],
        )
        .mount("/", routes![risk::get_risk_status, risk::update_risk])
        .mount(
            "/",
            routes![order::route::place_order, order::route::get_order],
        )
        .launch()
        .await
        .expect("Failed to launch rocket");
}

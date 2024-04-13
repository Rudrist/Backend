use rocket::form::{Form, Strict};
use rocket::http::{CookieJar, Status};
use rocket_db_pools::diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket_db_pools::Connection;

use crate::auth::user_center::get_logged_in_user_id;
use crate::db_lib::database;
use crate::db_lib::schema::{portfolios, risk_management};
use rocket::serde::json;
use rocket::serde::json::Value;
use rocket_db_pools::diesel::prelude::*;

#[get("/api/ledger")]
pub(crate) async fn get_ledger(
    mut accounts_db_conn: Connection<database::AccountsDb>,
    mut legders_db_conn: Connection<database::LedgersDb>,
    cookies: &CookieJar<'_>,
) -> Result<(Status, String), (Status, &'static str)> {
    // get user id
    let user_id = if let Some(user_id) = get_logged_in_user_id(cookies, &mut accounts_db_conn).await
    {
        user_id
    } else {
        return Err((
            Status::BadRequest,
            "Cannot fetch user id based on session token cookie or cookie crushed.",
        ));
    };

    // fetch ledgers
    let fetch_ledgers = ledgers::table
        .filter(ledgers::trader_account_id.eq(user_id))
        .select((
            ledgers::quote_currency,
            ledgers::base_currency,
            ledgers::quantity,
            ledgers::price,
            ledgers::send_pid,
            ledgers::receive_pid,
        ))
        .load::<(String, String, i64, i32, i32, i32)>(&mut legders_db_conn)
        .await;

    let ledgers = if let Ok(ledgers) = fetch_ledgers {
        ledgers
    } else {
        return Err((Status::BadRequest, "Ledgers not found"));
    };

    // return ledgers
    let mut ledgers_str = String::from("{\n  \"status\": \"successful\",\n  \"len\": ");
    ledgers_str.push_str(&ledgers.len().to_string());
    ledgers_str.push_str(",\n  \"data\": [\n");
    for (quote_currency, base_currency, quantity, price, send_pid, receive_pid) in ledgers {
        ledgers_str.push_str(&format!("    {{\n      \"quoteCurrency\": \"{}\",\n      \"baseCurrency\": \"{}\",\n      \"quantity\": {},\n      \"price\": {},\n      \"sendPid\": \"{}\",\n      \"receivePid\": \"{}\"\n    }},\n", quote_currency, base_currency, quantity, price, send_pid, receive_pid));
    }
    ledgers_str.pop();
    ledgers_str.push_str("  ]\n}");
    return Ok((Status::Ok, ledgers_str));
}

#[derive(FromForm)]
pub(crate) struct LedgerData<'r> {
    quoteCurrency: &'r str,
    baseCurrency: &'r str,
    quantity: i64,
    price: i32,
    sendPid: i32,
    receivePid: i32,
}

#[post("/api/ledger", data = "<ledger_data>")]
pub(crate) async fn send_ledger(
    ledger_data: Form<Strict<Value>>,
    mut accounts_db_conn: Connection<database::AccountsDb>,
    mut legders_db_conn: Connection<database::LedgersDb>,
    cookies: &CookieJar<'_>,
) -> Result<(Status, &'static str), (Status, &'static str)> {
    // check the existence of the ledger data
    let ledger_exist = ledgers::table
        .filter(ledgers::send_pid.eq(ledger_data.sendPid))
        .filter(ledgers::receive_pid.eq(ledger_data.receivePid))
        .select((
            ledgers::quote_currency,
            ledgers::base_currency,
            ledgers::quantity,
            ledgers::price,
            ledgers::send_pid,
            ledgers::receive_pid,
        ))
        .load::<(String, String, i64, i32, i32, i32)>(&mut legders_db_conn)
        .await;

    if let Ok(ledger_exist) = ledger_exist {
        if ledger_exist.len() > 0 {
            return Err((Status::BadRequest, "Ledger already exists"));
        }
    }

    // insert the ledger data
    let insert_ledger = diesel::insert_into(ledgers::table)
        .values((
            ledgers::quote_currency.eq(ledger_data.quoteCurrency),
            ledgers::base_currency.eq(ledger_data.baseCurrency),
            ledgers::quantity.eq(ledger_data.quantity),
            ledgers::price.eq(ledger_data.price),
            ledgers::send_pid.eq(ledger_data.sendPid),
            ledgers::receive_pid.eq(ledger_data.receivePid),
        ))
        .execute(&mut legders_db_conn)
        .await;

    if let Ok(_) = insert_ledger {
        return Ok((Status::Ok, "Ledger inserted successfully"));
    } else {
        return Err((Status::BadRequest, "Ledger insertion failed"));
    }
}

#[post("/api/ledger/accept", data = "<ledger_data>")]
pub(crate) async fn accept_ledger_request(
    ledger_data: Form<Strict<Value>>,
    mut accounts_db_conn: Connection<database::AccountsDb>,
    mut legders_db_conn: Connection<database::LedgersDb>,
    cookies: &CookieJar<'_>,
) -> Result<(Status, &'static str), (Status, &'static str)> {
    // check the existence of the ledger data
    let ledger_exist = ledgers::table
        .filter(ledgers::id.eq(ledger_data.id))
        .select((
            ledgers::quote_currency,
            ledgers::base_currency,
            ledgers::quantity,
            ledgers::price,
            ledgers::send_pid,
            ledgers::receive_pid,
        ))
        .load::<(String, String, i64, i32, i32, i32)>(&mut legders_db_conn)
        .await;

    if let Ok(ledger_exist) = ledger_exist {
        if ledger_exist.len() == 0 {
            return Err((Status::BadRequest, "Ledger not found"));
        }
    } else {
        return Err((Status::BadRequest, "Ledger not found"));
    }

    // update the ledger data
    // TODO: revise the price of the portfolio

    if let Ok(_) = update_ledger {
        return Ok((Status::Ok, "Ledger updated successfully"));
    } else {
        return Err((Status::BadRequest, "Ledger update failed"));
    }
}

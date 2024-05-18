use rocket::http::Status;
use rocket_db_pools::diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket_db_pools::Connection;

use crate::auth::validation::UserAuth;
use crate::db_lib::database;
use crate::db_lib::schema::{currencies, orders, portfolios, positions, quotations, trading_pairs};
use crate::order::bbgo;
use rocket::serde::json::{json, Json, Value};
use rocket_db_pools::diesel::prelude::*;
use serde::{Deserialize, Serialize};
async fn get_trading_pair(
    db_conn: &mut Connection<database::PgDb>,
    trading_pair_id: i32,
) -> Result<(String, String), &'static str> {
    let fetch_trading_pair = trading_pairs::table
        .filter(trading_pairs::id.eq(trading_pair_id))
        .select((
            trading_pairs::base_currency_id,
            trading_pairs::quote_currency_id,
        ))
        .first::<(i32, i32)>(db_conn)
        .await;
    if let Ok((base, quote)) = fetch_trading_pair {
        let base = get_currency(db_conn, base)
            .await
            .expect("Fetch base failed");
        let quote = get_currency(db_conn, quote)
            .await
            .expect("Fetch quote failed");
        return Ok((base, quote));
    } else {
        return Err("Fail to fetch trading pair");
    }
}
async fn get_currency(
    db_conn: &mut Connection<database::PgDb>,
    currency_id: i32,
) -> Result<String, &'static str> {
    let fetch_currency = currencies::table
        .filter(currencies::id.eq(currency_id))
        .select(currencies::code)
        .first::<String>(db_conn)
        .await;
    if let Ok(code) = fetch_currency {
        return Ok(code);
    } else {
        return Err("Fail to fetch currencies");
    }
}
async fn get_trading_pair_id(
    db_conn: &mut Connection<database::PgDb>,
    trading_pair: (&String, &String),
) -> Result<(i32, i32), &'static str> {
    let base = get_currency_id(db_conn, trading_pair.0)
        .await
        .expect("Fetch base failed");
    let quote = get_currency_id(db_conn, trading_pair.1)
        .await
        .expect("Fetch quote failed");

    let fetch_trading_pair = trading_pairs::table
        .filter(trading_pairs::base_currency_id.eq(base))
        .filter(trading_pairs::quote_currency_id.eq(quote))
        .select((
            trading_pairs::base_currency_id,
            trading_pairs::quote_currency_id,
        ))
        .first::<(i32, i32)>(db_conn)
        .await;
    if let Ok((base, quote)) = fetch_trading_pair {
        return Ok((base, quote));
    } else {
        return Err("Fail to fetch trading pair");
    }
}
async fn get_currency_id(
    db_conn: &mut Connection<database::PgDb>,
    currency_id: &String,
) -> Result<i32, &'static str> {
    let fetch_currency = currencies::table
        .filter(currencies::code.eq(currency_id))
        .select(currencies::id)
        .first::<i32>(db_conn)
        .await;
    if let Ok(id) = fetch_currency {
        return Ok(id);
    } else {
        return Err("Fail to fetch currencies");
    }
}
#[get("/api/order?<id>&<st>&<len>&<filter>")]
pub(crate) async fn get_order(
    id: i32,
    st: i32,
    len: i32,
    filter: String,
    mut db_conn: Connection<database::PgDb>,
    _user_auth: UserAuth,
) -> (Status, Value) {
    let user_id = _user_auth.user_id;

    let fetch_order = orders::table
        .inner_join(quotations::table.on(orders::quotation_id.eq(quotations::id)))
        .inner_join(positions::table.on(quotations::position_id.eq(positions::id)))
        .filter(positions::portfolio_id.eq(id))
        .select((
            orders::id,
            orders::buyin,
            orders::state,
            orders::trading_pair_id,
            orders::qty,
            orders::price,
        ))
        .offset(st.into())
        .limit(len.into())
        .load::<(i32, bool, i32, i32, i64, i64)>(&mut db_conn)
        .await
        .unwrap();

    let mut response_data: Vec<Value> = vec![];
    for (id, buyin, state, trading_pairs_id, qty, price) in fetch_order {
        let (base, quote) = get_trading_pair(&mut db_conn, trading_pairs_id)
            .await
            .unwrap();
        let state = match state {
            0 => "pending",
            1 => "success",
            2 => "fail",
            _ => "unknown",
        };
        response_data.push(json!({
            "id": id,
            "buyin": buyin,
            "state": state,
            "base": base,
            "quote": quote,
            "qty": qty,
            "price": price
        }));
    }
    let len = response_data.len();
    return (
        Status::Ok,
        json!({"status": "successful", "data":Value::from(response_data).to_string(), "len": len}),
    );
}

#[derive(Serialize, Deserialize)]
pub(crate) struct OrderData {
    base: String,
    quote: String,
    order_type: String,
    price: String,
    quantity: String,
}
#[post("/api/order", data = "<order_data>")]
pub(crate) async fn place_order(
    mut db_conn: Connection<database::PgDb>,
    _user_auth: UserAuth,
    order_data: Json<OrderData>,
) -> (Status, Value) {
    let user_id = _user_auth.user_id;
    let order_id = bbgo::handle_order(
        &order_data.base,
        &order_data.quote,
        &order_data.order_type,
        &order_data.price,
        &order_data.quantity,
    );
    let trading_pairs = get_trading_pair_id(&mut db_conn, (&order_data.base, &order_data.quote))
        .await
        .unwrap();
    let fetch_quotation = quotations::table
        .inner_join(positions::table.on(quotations::position_id.eq(positions::id)))
        .inner_join(portfolios::table.on(portfolios::id.eq(positions::portfolio_id)))
        .filter(quotations::base_currency_id.eq(trading_pairs.0))
        .filter(quotations::base_currency_id.eq(trading_pairs.0))
        .filter(portfolios::trader_account_id.eq(user_id))
        .select(quotations::id)
        .first::<i32>(&mut db_conn)
        .await
        .unwrap();
    let insert_order = rocket_db_pools::diesel::insert_into(orders::table)
        .values((
            orders::quotation_id.eq(fetch_quotation),
            orders::state.eq(0),
            orders::buyin.eq(order_data.order_type == "buy"),
            orders::price.eq(order_data.price.parse::<i64>().unwrap()),
            orders::qty.eq(order_data.quantity.parse::<i64>().unwrap()),
        ))
        .returning(orders::id)
        .get_result::<i32>(&mut db_conn)
        .await
        .unwrap();
    return (
        Status::Ok,
        json!({"status": "successful", "data": order_id}),
    );
}

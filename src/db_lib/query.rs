use crate::db_lib::database;
use crate::db_lib::schema::{currencies, trading_pairs, positions};
use rocket_db_pools::diesel::prelude::*;
use rocket_db_pools::Connection;
pub async fn get_trading_pair(
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
pub async fn get_currency(
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
pub async fn get_trading_pair_id(
    db_conn: &mut Connection<database::PgDb>,
    trading_pair: (&str, &str),
) -> Result<(i32, i32, i32), &'static str> {
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
            trading_pairs::id,
        ))
        .first::<(i32, i32, i32)>(db_conn)
        .await;
    if let Ok((base, quote, id)) = fetch_trading_pair {
        return Ok((base, quote, id));
    } else {
        return Err("Fail to fetch trading pair");
    }
}
pub async fn get_currency_id(
    db_conn: &mut Connection<database::PgDb>,
    currency_id: &str,
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

pub async fn get_position(
    db_conn: &mut Connection<database::PgDb>,
    base: &str, 
    quote: &str,
) -> Result<i32, &'static str> {
	let trading_pair ; 
	if let Ok((_, _, id)) = get_trading_pair_id(db_conn, (base, quote)).await {
		trading_pair = id;
	}else {
		return Err("Fail to fetch trading pair");
	}
	let fetch_position = positions::table
	.filter(positions::trading_pair_id.eq(trading_pair))
	.select(positions::id)
	.first::<i32>(db_conn)
	.await;

	if let Ok(id) = fetch_position {
		return Ok(id);
	}else{
		return Err("Fail to fetch position");
	}
}

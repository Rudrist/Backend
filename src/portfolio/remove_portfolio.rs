use ::diesel::ExpressionMethods;
use diesel::query_dsl::methods::{FilterDsl, SelectDsl};
use diesel::result::Error;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::serde::json::{json, Value};
use rocket_db_pools::diesel::prelude::RunQueryDsl;
use rocket_db_pools::{diesel, Connection};
use serde::{Deserialize, Serialize};



use crate::auth::validation::UserAuth;
use crate::db_lib::schema::{portfolio_balance, portfolios, trading_pairs, positions};
use crate::db_lib::database;

#[derive(Serialize, Deserialize)]
pub struct RemovePortfolioInfo<'r> {
    name: &'r str,
}

#[delete("/api/portfolio", data = "<remove_portfolio_info>")]
pub async fn remove_portfolio(
    remove_portfolio_info: Json<RemovePortfolioInfo<'_>>,
    mut db_conn: Connection<database::PgDb>,
    _user_auth: UserAuth,
) -> (Status, Value) {

    // ensure the user is logged in
    let _user_id = _user_auth.user_id;

    // get portfolio's id
    let portfolio_id_result: Result<i32, _> = portfolios::table
        .filter(portfolios::name.eq(remove_portfolio_info.name))
        .select(portfolios::id)
        .first(&mut db_conn)
        .await;

    let portfolio_id: i32 = match portfolio_id_result {
        Ok(id) => id,
        Err(_) => {
            return (
                Status::BadRequest,
                json!({"message": "The portfolio does not exist"}),
            );
        }
    };

    // delete portfolio_balance
    let portfolio_balance = diesel::delete(
        portfolio_balance::table.filter(portfolio_balance::portfolio_id.eq(portfolio_id)),
    )
    .execute(&mut db_conn)
    .await;

    match portfolio_balance {
        Ok(_) => (),
        Err(_) => {
            return (
                Status::InternalServerError,
                json!({"message": "Error deleting portfolio_balance"}),
            );
        }
    }

    let trading_pair_ids_result: Result<Vec<i32>, Error> = positions::table
        .filter(positions::portfolio_id.eq(portfolio_id))
        .select(positions::trading_pair_id)
        .load(&mut db_conn)
        .await;

    let trading_pair_ids  = match trading_pair_ids_result {
        Ok(ids) => ids,
        Err(_err) => {
            return (
                Status::InternalServerError,
                json!({"message": "Error fetching positions"}),
            );
        }
    };

    // delete positions
    let deleted_positions = diesel::delete(
        positions::table.filter(positions::portfolio_id.eq(portfolio_id))
    )
    .execute(&mut db_conn)
    .await;

    match deleted_positions {
        Ok(_) => (),
        Err(_) => {
            return (
                Status::InternalServerError,
                json!({"message": "Error deleting positions"}),
            );
        }
    }

    // delete trading_pairs
    for trading_pair_id in trading_pair_ids {
        let deleted_trading_pair = diesel::delete(
            trading_pairs::table.filter(trading_pairs::id.eq(trading_pair_id))
        )
        .execute(&mut db_conn)
        .await;

        match deleted_trading_pair {
            Ok(_) => (),
            Err(_) => {
                return (
                    Status::InternalServerError,
                    json!({"message": "Error deleting trading pair"}),
                );
            }
        }
    }

    // delete portfolio
    let portfolio = diesel::delete(portfolios::table.filter(portfolios::id.eq(portfolio_id)))
        .execute(&mut db_conn)
        .await;

    match portfolio {
        Ok(_) => {
            return (Status::Ok, json!({"status":"successful"}));
        } Err(_) => {
            return (
                Status::InternalServerError,
                json!({"message": "Error deleting portfolio"}),
            );
        }
    }   
}

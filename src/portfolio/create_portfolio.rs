use crate::auth::validation::UserAuth;
use crate::db_lib::database;
use crate::db_lib::query::*;
use crate::db_lib::schema::{portfolio_balance, portfolios, positions, quotations};
use ::diesel::ExpressionMethods;
use diesel::dsl::sql;
use diesel::sql_types::BigInt;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::serde::json::{json, Value};
use rocket_db_pools::diesel::prelude::RunQueryDsl;
use rocket_db_pools::{diesel, Connection};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AddPortfolioInfo<'r> {
    name: &'r str,
    position: Vec<String>,
}

#[post("/api/portfolio", data = "<add_portfolio_info>")]
pub async fn add_portfolio<'r>(
    add_portfolio_info: Json<AddPortfolioInfo<'r>>,
    mut db_conn: Connection<database::PgDb>,
    _user_auth: UserAuth,
) -> (Status, Value) {
    // ensure the user is logged in
    let user_id = _user_auth.user_id;

    // insert the portfolio data into the database
    let portfolio_result = rocket_db_pools::diesel::insert_into(portfolios::table)
        .values((
            portfolios::name.eq(add_portfolio_info.name.to_string()),
            portfolios::trader_account_id.eq(user_id),
            portfolios::portfolio_type.eq(2),
        ))
        .returning(portfolios::id)
        .get_result::<i32>(&mut db_conn)
        .await;

    let portfolio_id: i32 = match portfolio_result {
        Ok(value) => value,
        Err(_) => {
            return (
                Status::BadRequest,
                json!({"status":"error", "message": "Failed to insert into portfolios"}),
            );
        }
    };
    for pos in &add_portfolio_info.position {
        let base_id;
        let quote_id;
        let position: Vec<&str> = pos.split("/").collect();
        if let Ok((_base_id, _quote_id, _)) =
            get_trading_pair_id(&mut db_conn, (position[0], position[1])).await
        {
            base_id = _base_id;
            quote_id = _quote_id;
        } else {
            // TODO add rollback
            return (
                Status::BadRequest,
                json!({"status":"error", "message":"Position not found"}),
            );
        }
        let mut portfolio_balance_result =
            rocket_db_pools::diesel::insert_into(portfolio_balance::table)
                .values((
                    portfolio_balance::portfolio_id.eq(portfolio_id),
                    portfolio_balance::currency_id.eq(base_id),
                    portfolio_balance::quantity.eq(sql::<BigInt>("0")),
                ))
                .returning(portfolio_balance::id)
                .get_result::<i32>(&mut db_conn)
                .await;

        let _portfolio_balance_id: i32 = match portfolio_balance_result {
            Ok(value) => value,
            Err(_) => {
                return (
                    Status::BadRequest,
                    json!({"status":"error", "message": "Failed to insert base into portfolio_balance"}),
                );
            }
        };

        portfolio_balance_result = rocket_db_pools::diesel::insert_into(portfolio_balance::table)
            .values((
                portfolio_balance::portfolio_id.eq(portfolio_id),
                portfolio_balance::currency_id.eq(quote_id),
                portfolio_balance::quantity.eq(sql::<BigInt>("0")),
            ))
            .returning(portfolio_balance::id)
            .get_result::<i32>(&mut db_conn)
            .await;

        let _portfolio_balance_id: i32 = match portfolio_balance_result {
            Ok(value) => value,
            Err(_) => {
                return (
                    Status::BadRequest,
                    json!({"status":"error", "message": "Failed to insert quote into portfolio_balance"}),
                );
            }
        };
        let trading_pair_id = get_trading_pair_id(&mut db_conn, (position[0], position[1]))
            .await
            .unwrap()
            .2;
        // Insert into positions table
        let position_result = rocket_db_pools::diesel::insert_into(positions::table)
            .values((
                positions::trading_pair_id.eq(trading_pair_id),
                positions::portfolio_id.eq(portfolio_id),
            ))
            .returning(positions::id)
            .get_result::<i32>(&mut db_conn)
            .await;

        let _position_id: i32 = match position_result {
            Ok(value) => value,
            Err(_) => {
                return (
                    Status::BadRequest,
                    json!({"status":"error", "message": "Failed to insert into positions"}),
                );
            }
        };
        // Insert into quotation table
        let quotation_result = rocket_db_pools::diesel::insert_into(quotations::table)
            .values((
                quotations::quote_currency_id.eq(quote_id),
                quotations::position_id.eq(_position_id),
            ))
            .returning(quotations::id)
            .get_result::<i32>(&mut db_conn)
            .await;
        let _: i32 = match quotation_result {
            Ok(value) => value,
            Err(_) => {
                return (
                    Status::BadRequest,
                    json!({"status":"error", "message": "Failed to insert into quotations table"}),
                );
            }
        };
    }
    return (
        Status::Ok,
        json!({"status":"successful", "id": portfolio_id}),
    );
}

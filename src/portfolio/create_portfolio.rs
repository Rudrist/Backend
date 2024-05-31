use ::diesel::ExpressionMethods;
use diesel::dsl::sql;
use diesel::sql_types::BigInt;
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
pub struct Position<'r> {
    base_currency_id: &'r str,
    quote_currency_id: &'r str,
}

#[derive(Serialize, Deserialize
pub struct AddPortfolioInfo<'r> {
    name: &'r str,
    amount: &'r str,
    currency_id: &'r str,
    portfolio_type: &'r str,
    position: Vec<Position<'r>>,
}


#[post("/api/portfolio", data = "<add_portfolio_info>")]
pub async fn add_portfolio(
    add_portfolio_info: Json<AddPortfolioInfo<'_>>,
    mut db_conn: Connection<database::PgDb>,
    _user_auth: UserAuth,
) -> (Status, Value) {
    
    // ensure the user is logged in
    let user_id = _user_auth.user_id;

    let portfolio_type: i32 = match add_portfolio_info.portfolio_type.parse() {
        Ok(value) => value,
        Err(_) => {
            return (
                Status::BadRequest,
                json!({"message": "Invalid portfolio_type value"}),
            );
        }
    };

    // insert the portfolio data into the database
    let portfolio_result = rocket_db_pools::diesel::insert_into(portfolios::table)
        .values((
            portfolios::name.eq(add_portfolio_info.name.to_string()),
            portfolios::trader_account_id.eq(user_id),
            portfolios::portfolio_type.eq(portfolio_type),
        ))
        .returning(portfolios::id)
        .get_result::<i32>(&mut db_conn)
        .await;
    
    let portfolio_id: i32 = match portfolio_result {
        Ok(value) => value,
        Err(_) => {
            return (
                Status::BadRequest,
                json!({"message": "Failed to insert into portfolios"}),
            );
        }
    };

    let currency_id: i32 = match add_portfolio_info.currency_id.parse() {
        Ok(value) => value,
        Err(_) => {
            return (
                Status::BadRequest,
                json!({"message": "Invalid portfolio_type value"}),
            );
        }
    };

    let portfolio_balance_result = rocket_db_pools::diesel::insert_into(portfolio_balance::table)
        .values((
            portfolio_balance::portfolio_id.eq(portfolio_id),
            portfolio_balance::currency_id.eq(currency_id),
            portfolio_balance::quantity.eq(sql::<BigInt>(&add_portfolio_info.amount.to_string())),
        ))
        .returning(portfolio_balance::id)
        .get_result::<i32>(&mut db_conn)
        .await;

    let _portfolio_balance_id: i32 = match portfolio_balance_result {
        Ok(value) => value,
        Err(_) => {
            return (
                Status::BadRequest,
                json!({"message": "Failed to insert into portfolio_balance"}),
            );
        }
    };

    // Iterate over positions and insert into the corresponding tables
    for pos in &add_portfolio_info.position {
        let base_currency_id: i32 = match pos.base_currency_id.parse() {
            Ok(value) => value,
            Err(_) => {
                return (
                    Status::BadRequest,
                    json!({"message": "Invalid base_currency_id value"}),
                );
            }
        };

        let quote_currency_id: i32 = match pos.quote_currency_id.parse() {
            Ok(value) => value,
            Err(_) => {
                return (
                    Status::BadRequest,
                    json!({"message": "Invalid quote_currency_id value"}),
                );
            }
        };

        // Insert into trading_pairs table
        let trading_pair_result =  rocket_db_pools::diesel::insert_into(trading_pairs::table)
            .values((
                trading_pairs::base_currency_id.eq(base_currency_id),
                trading_pairs::quote_currency_id.eq(quote_currency_id),
            ))
            .returning(trading_pairs::id)
            .get_result::<i32>(&mut db_conn)
            .await;

        let trading_pair_id: i32 = match trading_pair_result {
            Ok(value) => value,
            Err(err) => {
                println!("Error: {:?}", err);
                return (
                    Status::BadRequest,
                    json!({"message": "Failed to insert into trading_pairs"}),
                );
            }
        };

        // Insert into positions table
        let position_result =  rocket_db_pools::diesel::insert_into(positions::table)
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
                    json!({"message": "Failed to insert into positions"}),
                );
            }
        };
    }
    return (Status::Ok, json!({"status":"successful"}));
}

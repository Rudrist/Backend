use ::diesel::ExpressionMethods;
use diesel::query_dsl::methods::SelectDsl;
use diesel::query_dsl::JoinOnDsl;
use rocket::http::Status;
use rocket::serde::json::{json, Value};
use rocket_db_pools::diesel::prelude::RunQueryDsl;
use rocket_db_pools::{diesel, Connection};
use serde::Serialize;

use crate::auth::validation::UserAuth;
use crate::db_lib::schema::{portfolio_balance, portfolios, positions, trading_pairs};
use crate::db_lib::database;

use std::collections::HashMap;

#[derive(Serialize)]
struct PortfolioData {
    portfolio: Vec<(String, i64, i32, Vec<(i32, i32)>)>,
    len: usize,
}


#[get("/api/portfolio")]
pub async fn get_portfolio_names(
    mut db_conn: Connection<database::PgDb>,
    _user_auth: UserAuth,
) -> (Status, Value) {
    // ensure the user is logged in
    let _user_id = _user_auth.user_id;

    // find all portfolios
    let portfolio_names_result: Result<Vec<String>, _> = portfolios::table
        .select(portfolios::name)
        .load(&mut db_conn)
        .await;

    match portfolio_names_result {
        Ok(portfolio_names) => {
            // HashMap to store portfolio information
            let mut portfolio_map: HashMap<String, (i64, i32, Vec<(i32, i32)>)> = HashMap::new();

            // find each portfolio's balance and positions
            for name in portfolio_names {
                use diesel::QueryDsl;
                // Query balance
                let balance_result: Result<(i64, i32), _> = SelectDsl::select(
                    diesel::QueryDsl::filter(portfolios::table, portfolios::name.eq(&name))
                        .inner_join(
                            portfolio_balance::table
                                .on(portfolios::id.eq(portfolio_balance::portfolio_id)),
                        ),
                    (portfolio_balance::quantity, portfolio_balance::currency_id),
                )
                .first(&mut db_conn)
                .await;

                // Query positions
                let position_result: Result<Vec<(i32, i32)>, _> = SelectDsl::select(
                    diesel::QueryDsl::filter(portfolios::table, portfolios::name.eq(&name))
                        .inner_join(
                            positions::table
                                .on(portfolios::id.eq(positions::portfolio_id)),
                        )
                        .inner_join(
                            trading_pairs::table
                                .on(positions::trading_pair_id.eq(trading_pairs::id))
                        ),
                    (trading_pairs::base_currency_id, trading_pairs::quote_currency_id)
                )
                .load(&mut db_conn)
                .await;

                match (balance_result, position_result) {
                    (Ok((balance, currency_id)), Ok(positions)) => {
                        portfolio_map.insert(name.clone(), (balance, currency_id, positions));
                    }
                    _ => {
                        return (
                            Status::InternalServerError,
                            json!({"message": "Failed to find the portfolio or portfolio balance"}),
                        );
                    }
                }
            }

            // Convert HashMap values to PortfolioData
            let mut portfolio_data = Vec::new();
            for (name, (balance, currency_id, positions)) in portfolio_map {
                portfolio_data.push((name, balance, currency_id, positions));
            }

            let num_portfolios = portfolio_data.len();
            let portfolio_data = PortfolioData {
                portfolio: portfolio_data,
                len: num_portfolios,
            };

            return (
                Status::Ok,
                json!({
                    "status": "successful",
                    "data": portfolio_data
                })
            );
        }
        Err(_) => {
            return (
                Status::InternalServerError,
                json!({"message": "Failed to find the portfolio"}),
            );
        }
    }
}


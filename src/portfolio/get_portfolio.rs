use ::diesel::ExpressionMethods;
use diesel::query_dsl::methods::SelectDsl;
use diesel::query_dsl::JoinOnDsl;
use rocket::http::Status;
use rocket::serde::json::{json, Value};
use rocket_db_pools::diesel::prelude::RunQueryDsl;
use rocket_db_pools::{diesel, Connection};

use crate::auth::validation::UserAuth;
use crate::db_lib::database;
use crate::db_lib::query::*;
use crate::db_lib::schema::{portfolio_balance, portfolios};

use std::collections::HashMap;

#[get("/api/portfolio")]
pub async fn get_portfolio_names(
    mut db_conn: Connection<database::PgDb>,
    _user_auth: UserAuth,
) -> (Status, Value) {
    // ensure the user is logged in
    let _user_id = _user_auth.user_id;

    // find all portfolios
    let portfolio_names_result: Result<Vec<(String, i32)>, _> = portfolios::table
        .select((portfolios::name, portfolios::id))
        .load(&mut db_conn)
        .await;

    match portfolio_names_result {
        Ok(portfolios) => {
            // HashMap to store portfolio information
            let mut portfolio_map: HashMap<String, (i32, Vec<Value>)> = HashMap::new();

            // find each portfolio's balance and positions
            for (name, id) in portfolios {
                use diesel::QueryDsl;
                // Query balance
                let balance_result: Result<Vec<(i64, i32)>, _> = SelectDsl::select(
                    diesel::QueryDsl::filter(portfolios::table, portfolios::name.eq(&name))
                        .inner_join(
                            portfolio_balance::table
                                .on(portfolios::id.eq(portfolio_balance::portfolio_id)),
                        ),
                    (portfolio_balance::quantity, portfolio_balance::currency_id),
                )
                .load(&mut db_conn)
                .await;

                match balance_result {
                    Ok(positions) => {
                        let mut re_positions = vec![];
                        for (a, b) in positions {
                            let currency = get_currency(&mut db_conn, b).await.unwrap();
                            re_positions.push(json!({"balance":a.to_string(), "symbol":currency}));
                        }
                        portfolio_map.insert(name.clone(), (id, re_positions));
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
            for (name, (id, positions)) in portfolio_map {
                portfolio_data.push(json!({"name":name, "id":id, "positions": positions}));
            }
            portfolio_data.sort_by(|a, b| a["id"].as_i64().cmp(&b["id"].as_i64()));
            let num_portfolios = portfolio_data.len();

            return (
                Status::Ok,
                json!({
                    "status": "successful",
                    "data": portfolio_data,
                    "len" : num_portfolios,
                }),
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

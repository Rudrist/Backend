use ::diesel::ExpressionMethods;
use diesel::query_dsl::methods::{FilterDsl, SelectDsl};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::serde::json::{json, Value};
use rocket_db_pools::diesel::prelude::RunQueryDsl;
use rocket_db_pools::{diesel, Connection};
use serde::{Deserialize, Serialize};

use crate::auth::validation::UserAuth;
use crate::db_lib::schema::{portfolio_balance, portfolios};
use crate::db_lib::database;

#[derive(Serialize, Deserialize)]
pub struct ChangePortfolioInfo<'r> {
    name: &'r str,
    amount: i32,
}


#[post("/change_portfolio", data = "<change_portfolio_info>")]
pub async fn change_portfolio(
    change_portfolio_info: Form<Strict<ChangePortfolioInfo<'_>>>,
    mut db_coon: Connection<database::PgDb>,
    cookies: &CookieJar<'_>,
) -> Result<Status, (Status, &'static str)> {
    // ensure the user is logged in
    let _user_id = _user_auth.user_id;

    // get portfolio's id
    let portfolio_id_result: Result<i32, _> = portfolios::table
        .filter(portfolios::name.eq(&change_portfolio_info.name))
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

    // update portfolio_balance
    let update_result = diesel::update(
        portfolio_balance::table.filter(portfolio_balance::portfolio_id.eq(portfolio_id)),
    )
    .set(portfolio_balance::quantity.eq(change_portfolio_info.amount as i64))
    .execute(&mut db_conn)
    .await;

    match update_result {
        Ok(updated_records) => {
            if updated_records == 0 {
                // not find matched portfolio
                return (
                    Status::BadRequest,
                    json!({"message": "Portfolio balance not found"}),
                );
            } else {
                return (Status::Ok, json!({"status":"successful"}));
            }
        }
        Err(_) => {
            return (
                Status::InternalServerError,
                json!({"message": "Failed to update portfolio balance"}),
            );
        }
    }

}

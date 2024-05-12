use rocket::form::{Form, Strict};
use rocket::http::{CookieJar, Status};
use rocket::response::Redirect;
use rocket_db_pools::{diesel, Connection};
use rocket_db_pools::diesel::prelude::RunQueryDsl;
use ::diesel::ExpressionMethods;
use diesel::query_dsl::methods::{FilterDsl, SelectDsl};
use diesel::dsl::sql;
use diesel::sql_types::BigInt;
use diesel::query_dsl::JoinOnDsl;
use rocket::serde::json::Json;

use crate::auth::user_center;
use crate::db_lib::{database, USER_COOKIE_NAME};
use crate::db_lib::schema::{sessions, accounts, portfolios, portfolio_balance};

#[get("/get_portfolio_names")]
pub(crate) async fn get_portfolio_names(
    mut accounts_db_coon: Connection<database::AccountsDb>, 
    cookies: &CookieJar<'_>
) -> Result<Json<(Vec<(String, i64, i32)>, usize)>, Status> {
    // ensure the user is logged in
    if user_center::get_logged_in_user_id(cookies, &mut accounts_db_coon).await.is_none() {
        return Err(Status::BadRequest);
    }

    // find all portfolios
    let portfolio_names_result: Result<Vec<String>, _> = portfolios::table
        .select(portfolios::name)
        .load(&mut accounts_db_coon)
        .await;

    match portfolio_names_result {
        Ok(portfolio_names) => {
            // find each portfolio's balance 
            let mut portfolio_balances = Vec::new();
            for name in portfolio_names {
                use diesel::QueryDsl;

                let balance_result: Result<(i64, i32), _> = SelectDsl::select(
                    diesel::QueryDsl::filter(
                        portfolios::table,
                        portfolios::name.eq(&name)
                    )
                    .inner_join(
                        portfolio_balance::table.on(portfolios::id.eq(portfolio_balance::portfolio_id))
                    ),
                    (portfolio_balance::quantity, portfolio_balance::currency_id)
                )
                .first(&mut accounts_db_coon)
                .await;
            
                match balance_result {
                    Ok((balance, currency_id)) => portfolio_balances.push((name, balance, currency_id)),
                    Err(_) => return Err(Status::InternalServerError), 
                } 
            }
            let num_portfolios = portfolio_balances.len();
            Ok(Json((portfolio_balances, num_portfolios)))
        },
        Err(_) => Err(Status::InternalServerError), 
    }
}

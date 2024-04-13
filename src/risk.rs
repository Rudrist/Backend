use rocket::form::{Form, Strict};
use rocket::http::{CookieJar, Status};
use rocket_db_pools::diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket_db_pools::Connection;

use crate::auth::user_center::get_logged_in_user_id;
use crate::db_lib::database;
use crate::db_lib::schema::{portfolios, risk_management};
use rocket_db_pools::diesel::prelude::*;

#[get("/api/risk")]
pub(crate) async fn get_risk_status(
    mut accounts_db_coon: Connection<database::AccountsDb>,
    mut risk_management_db_conn: Connection<database::RiskManagementDb>,
    cookies: &CookieJar<'_>,
) -> Result<(Status, String), (Status, &'static str)> {
    // get user id
    let user_id = if let Some(user_id) = get_logged_in_user_id(cookies, &mut accounts_db_coon).await
    {
        user_id
    } else {
        return Err((
            Status::BadRequest,
            "Cannot fetch user id based on session token cookie or cookie crushed.",
        ));
    };

    // fetch risk status
    let fetch_risk_status = risk_management::table
        .inner_join(portfolios::table)
        .filter(portfolios::trader_account_id.eq(user_id))
        .select((
            risk_management::risk_type,
            risk_management::valid,
            risk_management::pnl,
            risk_management::position,
            risk_management::portfolio_id,
        ))
        .load::<(String, bool, i64, i32, i32)>(&mut risk_management_db_conn)
        .await;

    let risk_status = if let Ok(risk_status) = fetch_risk_status {
        risk_status
    } else {
        return Err((Status::BadRequest, "Risk staqtus not found"));
    };

    // return risk status
    let mut risk_status_str = String::from("{\n  \"success\": true,\n  \"data\": [\n");
    for (risk_type, valid, pnl, position, portfolio_id) in risk_status {
        risk_status_str.push_str(&format!("    {{\n      \"type\": \"{}\",\n      \"on\": {},\n      \"pnl\": {},\n      \"position\": \"{}\",\n      \"pid\": \"{}\"\n    }},\n", risk_type, valid, pnl, position, portfolio_id));
    }
    risk_status_str.pop();
    risk_status_str.push_str("  ]\n}");
    return Ok((Status::Ok, risk_status_str));
}

#[derive(FromForm)]
pub(crate) struct RiskData<'r> {
    risk_type: &'r str,
    valid: bool,
    pnl: i64,
    position: i32,
    portfolio_id: i32,
}

#[post("/api/risk", data = "<risk_data>")]
pub(crate) async fn update_risk(
    risk_data: Form<Strict<RiskData<'_>>>,
    mut accounts_db_coon: Connection<database::AccountsDb>,
    mut portfolios_db_conn: Connection<database::PortfolioDb>,
    mut risk_management_db_conn: Connection<database::RiskManagementDb>,
    cookies: &CookieJar<'_>,
) -> Result<(Status, &'static str), (Status, &'static str)> {
    // get user id
    let user_id = if let Some(user_id) = get_logged_in_user_id(cookies, &mut accounts_db_coon).await
    {
        user_id
    } else {
        return Err((
            Status::BadRequest,
            "Cannot fetch user id based on session token cookie or cookie crushed.",
        ));
    };

    // check the ownership
    let fetch_user_id = portfolios::table
        .filter(portfolios::id.eq(risk_data.portfolio_id))
        .select(portfolios::trader_account_id)
        .first::<i32>(&mut portfolios_db_conn)
        .await;

    match fetch_user_id {
        Ok(owner_id) if owner_id == user_id => {
            // update database
            let update_risk_info = diesel::update(
                risk_management::table
                    .filter(risk_management::portfolio_id.eq(risk_data.portfolio_id)),
            )
            .set((
                risk_management::risk_type.eq(risk_data.risk_type),
                risk_management::valid.eq(risk_data.valid),
                risk_management::pnl.eq(risk_data.pnl),
                risk_management::position.eq(risk_data.position),
            ))
            .execute(&mut risk_management_db_conn)
            .await;

            match update_risk_info {
                Ok(_) => Ok((
                    Status::Accepted,
                    "The risk management data is successfully updated.",
                )),
                Err(_) => Err((
                    Status::BadRequest,
                    "Fail to update the risk management data.",
                )),
            }
        }
        Ok(_) => Err((Status::Forbidden, "You do not own this portfolio.")),
        Err(_) => Err((Status::BadRequest, "Portfolio not found or database error.")),
    }
}

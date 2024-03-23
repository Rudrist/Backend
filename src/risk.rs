use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::http::{CookieJar, Status};
use rocket_db_pools::Connection;

use crate::auth::user_center::get_logged_in_user_id;
use crate::db_lib::database;
use crate::db_lib::schema::{portfolios, risk_management};

#[get("/api/risk")]
pub(crate) async fn get_risk_status(
    mut accounts_db_coon: Connection<database::AccountsDb>,
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

    // fetch risk status
    let fetch_risk_status = risk_management::table
        .inner_join(portfolios::table)
        .select((
            risk_management::risk_type,
            risk_management::valid,
            risk_management::pnl,
            risk_management::position,
            risk_management::portfolio_id,
        ))
        .filter(portfolios::trader_account_id.eq(user_id))
        .load::<(String, bool, i64, i32, i32)>(&mut risk_management_db_conn);

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

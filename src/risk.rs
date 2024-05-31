use rocket::http::{CookieJar, Status};
use rocket_db_pools::diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket_db_pools::Connection;
// use rudrist_backend::db_lib::schema::risk_management::position;

use crate::auth::user_center::get_logged_in_user_id;
use crate::auth::validation::UserAuth;
use crate::db_lib::database;
use crate::db_lib::query::*;
use crate::db_lib::schema::{portfolios, risk_management};
use rocket::serde::json::Json;
use rocket::serde::json::{json, Value};
use serde::{Deserialize, Serialize};
#[get("/api/risk")]
pub async fn get_risk_status(
    mut db_conn: Connection<database::PgDb>,
    // cookies: &CookieJar<'_>,
    _user_auth: UserAuth,
) -> (Status, Value) {
    // get user id
    // let user_id = if let Some(user_id) = get_logged_in_user_id(cookies, &mut db_conn).await
    // {
    //     user_id
    // } else {
    //     return (
    //         Status::BadRequest,
    //         json!({"status":"error", "message":"Cannot fetch user id based on session token cookie or cookie crushed."})
    //     );
    // };
    let user_id = _user_auth.user_id;

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
        .load::<(String, bool, i64, i32, i32)>(&mut db_conn)
        .await;

    let risk_status = if let Ok(risk_status) = fetch_risk_status {
        risk_status
    } else {
        return (
            Status::BadRequest,
            json!({"status":"error", "message":"Risk staqtus not found"}),
        );
    };

    // return risk status
    let mut risk_data: Vec<Value> = vec![];
    let len = risk_status.len();
    for (risk_type, valid, pnl, position, portfolio_id) in risk_status {
        risk_data.push(json!({"type": risk_type, "on": valid, "pnl": pnl, "position": position, "portfolio_id": portfolio_id}));
    }
    return (
        Status::Ok,
        json!({"status": "successful", "data":Value::from(risk_data).to_string(), "len": len}),
    );
}

#[derive(Serialize, Deserialize)]
pub struct RiskData<'r> {
    risk_type: &'r str,
    on: bool,
    pnl: i64,
    position: &'r str,
    pid: i32,
}

#[post("/api/risk", data = "<risk_data>")]
pub async fn update_risk(
    risk_data: Json<RiskData<'_>>,
    mut db_conn: Connection<database::PgDb>,
    cookies: &CookieJar<'_>,
) -> (Status, Value) {
    // check the existence of the risk management data
    let position_id;
    let position:Vec<&str> = risk_data.position.split("/").collect();
    if let Ok(id) = get_position(&mut db_conn, position[0], position[1]).await{
        position_id = id;
    }else{
        return (
            Status::BadRequest,
            json!({"status":"error", "message":"Position not found"}),
        );
    }
    let risk_management_exist = risk_management::table
        .filter(risk_management::portfolio_id.eq(risk_data.pid))
        .select((
            risk_management::risk_type,
            risk_management::valid,
            risk_management::pnl,
            risk_management::position,
            risk_management::portfolio_id,
        ))
        .first::<(String, bool, i64, i32, i32)>(&mut db_conn)
        .await;

    if let Err(_) = risk_management_exist {
        // if the risk management data does not exist, insert it
        let insert_risk_info = diesel::insert_into(risk_management::table)
            .values((
                risk_management::portfolio_id.eq(risk_data.pid),
                risk_management::risk_type.eq(risk_data.risk_type),
                risk_management::valid.eq(risk_data.on),
                risk_management::pnl.eq(risk_data.pnl),
                risk_management::position.eq(position_id),
            ))
            .execute(&mut db_conn)
            .await;

        if let Err(_) = insert_risk_info {
            return (
                Status::BadRequest,
                json!({"status":"error", "message": "Database error."}),
            );
        } else {
            return (
                Status::Ok,
                json!({"status":"successful", "message":"Risk management data inserted."}),
            );
        }
    } else {
        // if the risk management data exists
        // fisrt, check the ownership
        let user_id = if let Some(user_id) = get_logged_in_user_id(cookies, &mut db_conn).await {
            user_id
        } else {
            return (
                Status::BadRequest,
                json!("Cannot fetch user id based on session token cookie or cookie crushed."),
            );
        };

        let fetch_user_id = portfolios::table
            .filter(portfolios::id.eq(risk_data.pid))
            .select(portfolios::trader_account_id)
            .first::<i32>(&mut db_conn)
            .await;

        match fetch_user_id {
            Ok(owner_id) if owner_id != user_id => {
                return (
                    Status::Forbidden,
                    json!({"status":"error", "message":"You do not own this portfolio."}),
                );
            }
            Ok(_) => {}
            Err(_) => {
                return (
                    Status::BadRequest,
                    json!({"status":"error", "message":"Portfolio not found or database error."}),
                );
            }
        }

        // then, update the database
        let update_risk_info = diesel::update(
            risk_management::table.filter(risk_management::portfolio_id.eq(risk_data.pid)),
        )
        .set((
            risk_management::risk_type.eq(risk_data.risk_type),
            risk_management::valid.eq(risk_data.on),
            risk_management::pnl.eq(risk_data.pnl),
            risk_management::position.eq(position_id),
        ))
        .execute(&mut db_conn)
        .await;

        if let Err(_) = update_risk_info {
            return (
                Status::BadRequest,
                json!({"status":"error", "message":"Database error."}),
            );
        } else {
            return (
                Status::Ok,
                json!({"status":"error", "message":"Risk management data updated."}),
            );
        }
    }
}

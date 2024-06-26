use ::diesel::ExpressionMethods;
use diesel::query_dsl::methods::{FilterDsl, SelectDsl};
use diesel::result::Error;
use rocket::http::Status;
use rocket::serde::json::{json, Value};
use rocket_db_pools::diesel::prelude::RunQueryDsl;
use rocket_db_pools::{diesel, Connection};

use crate::auth::validation::UserAuth;
use crate::db_lib::database;
use crate::db_lib::schema::{orders, portfolio_balance, portfolios, positions, quotations};

#[delete("/api/portfolio?<name>")]
pub async fn remove_portfolio(
    name: String,
    mut db_conn: Connection<database::PgDb>,
    _user_auth: UserAuth,
) -> (Status, Value) {
    // ensure the user is logged in
    let _user_id = _user_auth.user_id;

    // get portfolio's id
    let portfolio_id_result: Result<i32, _> = portfolios::table
        .filter(portfolios::name.eq(name))
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

    let position_ids_result: Result<Vec<i32>, Error> = positions::table
        .filter(positions::portfolio_id.eq(portfolio_id))
        .select(positions::id)
        .load(&mut db_conn)
        .await;

    let positions_ids = match position_ids_result {
        Ok(ids) => ids,
        Err(_err) => {
            return (
                Status::InternalServerError,
                json!({"message": "Error fetching positions"}),
            );
        }
    };
    let delete_order = diesel::delete(orders::table.filter(orders::portfolio_id.eq(portfolio_id)))
        .execute(&mut db_conn)
        .await;
    match delete_order {
        Ok(_) => (),
        Err(_) => {
            return (
                Status::InternalServerError,
                json!({"message": "Error deleting order"}),
            );
        }
    }
    // delete quotation
    for position_id in positions_ids {
        let deleted_quotation =
            diesel::delete(quotations::table.filter(quotations::position_id.eq(position_id)))
                .execute(&mut db_conn)
                .await;
        match deleted_quotation {
            Ok(_) => (),
            Err(_) => {
                return (
                    Status::InternalServerError,
                    json!({"message": "Error deleting quotation"}),
                );
            }
        }
    }

    // delete positions
    let deleted_positions =
        diesel::delete(positions::table.filter(positions::portfolio_id.eq(portfolio_id)))
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

    // delete portfolio
    let portfolio = diesel::delete(portfolios::table.filter(portfolios::id.eq(portfolio_id)))
        .execute(&mut db_conn)
        .await;

    match portfolio {
        Ok(_) => {
            return (Status::Ok, json!({"status":"successful"}));
        }
        Err(_) => {
            return (
                Status::InternalServerError,
                json!({"message": "Error deleting portfolio"}),
            );
        }
    }
}

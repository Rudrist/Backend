use rocket::form::{Form, Strict};
use rocket::http::Status;
use rocket::response::Redirect;
use rocket_db_pools::diesel::prelude::RunQueryDsl;
use rocket_db_pools::Connection;
use pbkdf2::password_hash::PasswordHasher;
use ::diesel::ExpressionMethods;

use crate::db_lib::database;
use crate::db_lib::schema;

// The signup info of the user. Simple constraints are checked in the front end (html).
#[derive(FromForm)]
pub(crate) struct SignupInfo<'r> {
    user_name: &'r str,
    user_email: &'r str,
    user_password: &'r str,
    confirm_password: &'r str
}

// TODO, signup is available only when not logged in
// if signup sucessfully, redirect to login page. (It won't log in automatically)
// Otherwise, return Status::BadRequest and a string indicating the error. (It is not fancy at all :< )
#[post("/api/auth/register", data = "<signup_info>")]
pub(crate) async fn signup(
    signup_info: Form<Strict<SignupInfo<'_>>>, 
    mut accounts_db_coon: Connection<database::AccountsDb>
) -> Result<Status, (Status, &'static str)> {

    // confirm the password
    if signup_info.user_password != signup_info.confirm_password {
        return Err((Status::BadRequest, "The password doesn't match."));
    }

    // hash the password
    let salt = pbkdf2::password_hash::SaltString::generate(&mut rand_core::OsRng);
    let password_hash = pbkdf2::Pbkdf2.hash_password(signup_info.user_password.as_bytes(), &salt);
    let hashed_password = if let Ok(_password) = password_hash {
        _password.to_string()
    } else {
        return Err((Status::BadRequest, "The password is invalid."))
    };

    // inser the signup user data into the database
    let signup_user_id = rocket_db_pools::diesel::insert_into(schema::accounts::table)
        .values((
            schema::accounts::username.eq(signup_info.user_name.to_string()),
            schema::accounts::email.eq(signup_info.user_email.to_string()),
            schema::accounts::password.eq(&hashed_password),
        ))
        .execute(&mut accounts_db_coon).await;
    
    // No need to create default portfolio.
    // match signup_user_id{
    //     Ok(id) => {
    //         let i32_id: i32 = id as i32;
    //         // inser the main portfolio data into the database
    //         let main_portfolio_id = rocket_db_pools::diesel::insert_into(schema::portfolios::table)
    //         .values((
    //             schema::portfolios::name.eq(format!("{} main account", signup_info.user_name)),
    //             schema::portfolios::trader_account_id.eq(i32_id),
    //             schema::portfolios::portfolio_type.eq(2),
    //         ))
    //         .execute(&mut accounts_db_coon).await;
            
    //         let main_portfolio_balance = rocket_db_pools::diesel::insert_into(schema::portfolio_balance::table)
    //         .values((
    //             schema::portfolio_balance::quantity.eq(0),
    //         ))
    //         .execute(&mut accounts_db_coon).await;
        
    //         return Ok(Status::Ok);
    //     }
    //     Err(err) => {
    //         eprintln!("{:?}", err);
    //         return Err((Status::BadRequest, "Account already exist."));
    //     }
    // }

    
    match signup_user_id {
        Ok(_) => {
            return Ok(Status::Ok);
        }
        Err(_) => {
            return Err((Status::BadRequest, "Account already exist."));
        }
    }
}
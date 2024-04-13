use diesel::ExpressionMethods;
use rand_core::RngCore;
use rocket::http::Status;
use rocket_db_pools::diesel::RunQueryDsl;
use rocket_db_pools::Connection;
use std::sync::{Arc, Mutex};

use crate::db_lib::database;
use crate::db_lib::schema;

type Random = Arc<Mutex<rand_chacha::ChaCha8Rng>>;

#[derive(Clone, Copy)]
pub(crate) struct SessionToken(u128);

impl SessionToken {
    pub fn generate_new(random: Random) -> Self {
        let mut u128_pool = [0u8; 16];
        random.lock().unwrap().fill_bytes(&mut u128_pool);
        Self(u128::from_le_bytes(u128_pool))
    }

    pub fn into_cookie_value(self) -> String {
        // TODO Opportunity for a smaller format that is still a valid cookie value
        self.0.to_string()
    }

    pub fn into_database_value(self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }

    pub fn to_token(value: String) -> Self {
        Self(value.parse::<u128>().unwrap())
    }
}

// generate a session token, insert it into database, and return it if successfully otherwise return Status::BadRequest
pub(crate) async fn new_session(
    random: Random,
    user_id: i32,
    mut accounts_db_conn: &mut Connection<database::AccountsDb>,
) -> Result<SessionToken, (Status, &'static str)> {
    let session_token = SessionToken::generate_new(random);
    let insert_session = rocket_db_pools::diesel::insert_into(schema::sessions::table)
        .values((
            schema::sessions::user_id.eq(user_id),
            schema::sessions::session_token.eq(session_token.into_database_value()),
        ))
        .execute(&mut accounts_db_conn)
        .await;

    if let Ok(_) = insert_session {
        return Ok(session_token);
    } else {
        return Err((Status::BadRequest, "Fail to generate new session"));
    }
}

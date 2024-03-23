pub mod schema;
pub mod models;
pub mod database;


use std::sync::{Arc, Mutex};
use rand_chacha::ChaCha8Rng;


pub const USER_COOKIE_NAME: &str = "user_token";
// const COOKIE_MAX_AGE: &str = "9999999";

// this structure is used to help session (the name is not fancy at all)
pub struct RAND {
    pub random: Arc<Mutex<ChaCha8Rng>>
}
mod authentication;
mod errors;
mod utils;

use std::sync::{Arc, Mutex};

use axum::{
    extract::{Extension, Multipart, Path},
    middleware,
    response::{Html, IntoResponse, Redirect},
    routing::{any, get, post},
    Router
};
use http::Response;

use authentication::{auth, delete_user, login, signup, send_email, change_password, AuthState};
use errors::{LoginError, NoUser, NotLoggedIn, SignupError};
use pbkdf2::password_hash::rand_core::OsRng;
use rand_chacha::ChaCha8Rng;
use rand_core::{RngCore, SeedableRng};
use shuttle_service::error::CustomError;
use shuttle_axum::ShuttleAxum;
use sqlx::Executor;
use tera::{Context, Tera};
use utils::*;
use chrono::{DateTime, Utc};
use urlencoding::decode;


use sqlx::PgPool;

type Templates = Arc<Tera>;
type Database = sqlx::PgPool;
type Random = Arc<Mutex<ChaCha8Rng>>;

const USER_COOKIE_NAME: &str = "user_token";
const COOKIE_MAX_AGE: &str = "9999999";

// Define a struct to extract query parameters


#[shuttle_runtime::main]
async fn server(
  #[shuttle_shared_db::Postgres] pool: PgPool
) -> ShuttleAxum {
pool.execute(include_str!("../schema.sql"))
          .await
          .map_err(CustomError::new)?;
 
      Ok(get_router(pool).into())
}


pub fn get_router(database: Database) -> Router {
    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("base.html", include_str!("../templates/base.html")),
        ("index", include_str!("../templates/index.html")),
        ("signup", include_str!("../templates/signup.html")),
        ("login", include_str!("../templates/login.html")),
        ("users", include_str!("../templates/users.html")),
        ("user", include_str!("../templates/user.html")),
        ("forgotpassword", include_str!("../templates/forgotpassword.html")),
        ("resetpassword", include_str!("../templates/resetpassword.html")),
        ("verification", include_str!("../templates/verification.html"))
    ])
    .unwrap();

    let middleware_database = database.clone();
    let random = ChaCha8Rng::seed_from_u64(OsRng.next_u64());

    Router::new()
        .route("/", get(index))
        .route("/signup", get(get_signup).post(post_signup))
        .route("/login", get(get_login).post(post_login))
        .route("/login/forgotpassword",get(get_forgotpassword).post(post_forgotpassword))
        .route("/login/forgotpassword/:username/:resettoken/:expiration_timestamp", get(get_resetpassword).post(post_resetpassword))
        .route("/login/forgotpassword/verification", get(get_verifiaction).post(post_forgotpassword))
        .route("/logout", post(logout_response))
        .route("/delete", post(post_delete))
        .route("/me", get(me))
        .route("/user/:username", get(user))
        .route("/users", get(users))
        .route("/styles.css", any(styles))
        .layer(middleware::from_fn(move |req, next| {
            auth(req, next, middleware_database.clone())
        }))
        .layer(Extension(Arc::new(tera)))
        .layer(Extension(database))
        .layer(Extension(Arc::new(Mutex::new(random))))
}

async fn index(
    Extension(current_user): Extension<AuthState>,
    Extension(templates): Extension<Templates>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("logged_in", &current_user.logged_in());
    context.insert("home_screen", &true);
    Html(templates.render("index", &context).unwrap())
}

async fn user(
    Path(username): Path<String>,
    Extension(mut auth_state): Extension<AuthState>,
    Extension(database): Extension<Database>,
    Extension(templates): Extension<Templates>,
) -> impl IntoResponse {
    const QUERY: &str = "SELECT username FROM users WHERE username = $1;";

    if !auth_state.logged_in() {
        return Ok(Html(templates.render("login", &Context::new()).unwrap()));
    }

    let user: Option<(String,)> = sqlx::query_as(QUERY)
        .bind(&username)
        .fetch_optional(&database)
        .await
        .unwrap();

    if let Some((username,)) = user {
        let user_is_self = auth_state
            .get_user()
            .await
            .map(|logged_in_user| logged_in_user.username == username)
            .unwrap_or_default();

        if user_is_self {
            let mut context = Context::new();
            context.insert("username", &username);
            context.insert("is_self", &user_is_self);
            Ok(Html(templates.render("user", &context).unwrap()))
        } else {
            // Users not having right to access, return to login page
            return Ok(Html(templates.render("login", &Context::new()).unwrap()));
        }

    } else {
        Err(error_page(&NoUser(username)))
    }
}

async fn get_resetpassword(
    Path((username, resettoken, expiration_timestamp)): Path<(String, String, String)>,
    Extension(templates): Extension<Arc<Tera>>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("username", &username);
    context.insert("resettoken", &resettoken);

    // Convert expiration_timestamp to DateTime<Utc>
    let expiration_timestamp_decoded = decode(&expiration_timestamp).unwrap_or_default();
    let expiration_timestamp_utc: DateTime<Utc> = expiration_timestamp_decoded.parse().unwrap();
    context.insert("expiration_timestamp", &expiration_timestamp_utc.to_string());

    if Utc::now() > expiration_timestamp_utc {
        // The link has expired
        return Html(templates.render("forgotpassword", &Context::new()).unwrap());
    }

    Html(templates.render("resetpassword", &context).unwrap())
}

async fn get_signup(Extension(templates): Extension<Templates>) -> impl IntoResponse {
    Html(templates.render("signup", &Context::new()).unwrap())
}

async fn get_login(Extension(templates): Extension<Templates>) -> impl IntoResponse {
    Html(templates.render("login", &Context::new()).unwrap())
}

async fn get_forgotpassword(Extension(templates): Extension<Templates>) -> impl IntoResponse {
    Html(templates.render("forgotpassword", &Context::new()).unwrap())
}

async fn get_verifiaction(Extension(templates): Extension<Templates>) -> impl IntoResponse {
    Html(templates.render("verification", &Context::new()).unwrap())
}

async fn post_signup(
    Extension(database): Extension<Database>,
    Extension(random): Extension<Random>,
    multipart: Multipart,
) -> impl IntoResponse {
    let data = parse_multipart(multipart)
        .await
        .map_err(|err| error_page(&err))?;

    if let (Some(username), Some(password), Some(confirm_password), Some(email)) = (
        data.get("username"),
        data.get("password"),
        data.get("confirm_password"),
        data.get("email"),
    ) {

        if password != confirm_password {
            return Err(error_page(&SignupError::PasswordsDoNotMatch));
        }
        
        match signup(&database, random, username, password, email).await {
            Ok(session_token) => Ok(login_response(session_token)),
            Err(error) => Err(error_page(&error)),
        }
    } else {
        Err(error_page(&SignupError::MissingDetails))
    }
}

async fn post_login(
    Extension(database): Extension<Database>,
    Extension(random): Extension<Random>,
    multipart: Multipart,
) -> impl IntoResponse {
    let data = parse_multipart(multipart)
        .await
        .map_err(|err| error_page(&err))?;

    if let (Some(username), Some(password)) = (data.get("username"), data.get("password")) {
        match login(&database, random, username, password).await {
            Ok(session_token) => Ok(login_response(session_token)),
            Err(err) => Err(error_page(&err)),
        }
    } else {
        Err(error_page(&LoginError::MissingDetails))
    }
}

async fn post_forgotpassword(
    Extension(database): Extension<Database>,
    multipart: Multipart,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let data = parse_multipart(multipart)
        .await
        .map_err(|err| error_page(&err))?;

    if let Some(username) = data.get("username") {
        let reset_token = OsRng.next_u64();
        match send_email(&database, username, &reset_token).await {
            Ok(_) => Ok(Redirect::to(&format!("/login/forgotpassword/verification"))),
            Err(err) => Err(error_page(&err)),
        }
    } else {
        Err(error_page(&LoginError::MissingDetails))
    }
}

async fn post_resetpassword(
    Path((username, _resettoken, _expiration_timestamp)): Path<(String, String, String)>,
    Extension(database): Extension<Database>,
    multipart: Multipart,
) -> Result<impl IntoResponse, impl IntoResponse>{
    
    let data = parse_multipart(multipart)
        .await
        .map_err(|err| error_page(&err))?;

    if let Some(password) = data.get("password") {
        match change_password(&database,  &username, password).await {
            Ok(_) => Ok(Redirect::to(&format!("/login"))),
            Err(error) => Err(error_page(&error)),
        }
    } else {
        Err(error_page(&SignupError::MissingDetails))
    }
}

async fn post_delete(Extension(current_user): Extension<AuthState>) -> impl IntoResponse {
    if !current_user.logged_in() {
        return Err(error_page(&NotLoggedIn));
    }

    delete_user(current_user).await;

    Ok(logout_response().await)
}

async fn styles() -> impl IntoResponse {
    Response::builder()
        .status(http::StatusCode::OK)
        .header("Content-Type", "text/css")
        .body(include_str!("../public/styles.css").to_owned())
        .unwrap()
}

async fn me(
    Extension(mut current_user): Extension<AuthState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    if let Some(user) = current_user.get_user().await {
        Ok(Redirect::to(&format!("/user/{}", user.username)))
    } else {
        Err(error_page(&NotLoggedIn))
    }
}

async fn users(
    Extension(database): Extension<Database>,
    Extension(templates): Extension<Templates>,
) -> impl IntoResponse {
    const QUERY: &str = "SELECT username FROM users LIMIT 100;";

    let users: Vec<(String,)> = sqlx::query_as(QUERY).fetch_all(&database).await.unwrap();

    // This should be a no op right :)
    let users = users.into_iter().map(|(value,)| value).collect::<Vec<_>>();

    let mut context = Context::new();
    context.insert("users", &users);

    Html(templates.render("users", &context).unwrap())
}
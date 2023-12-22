# Authentication Module

This authentication module is built on an Axum web server with Tera templates, using a PostgreSQL database with sqlx, and hosted on Shuttle.

## Features

- Implementation of login, logout, and password recovery.
- Includes a test web page.

## How to Use

To utilize the authentication features, use the `Extension` provided by the Axum framework to extract the `AuthState`.

Extension(current_user): Extension<AuthState>

You can then call the logged_in() function to check if a user is currently logged in:
s
current_user.logged_in()


## References and Test Environment
Please ensure that you have set up the Shuttle environment. For instructions, refer to the following link:

[Building an authentication system in Rust using session tokens](https://www.shuttle.rs/blog/2022/08/11/authentication-tutorial)

Special thanks to the article above for inspiration and guidance in creating this system.
# Authentication Module

This authentication module is built on an Axum web server with Tera templates, using a PostgreSQL database with sqlx, and hosted on Shuttle.

## Features

- Implementation of login, logout, and password recovery.
- Includes a test web page.

## How to Use

To utilize the authentication features, use the `Extension` provided by the Axum framework to extract the `AuthState`.

```rust
Extension(current_user): Extension<AuthState>
```

You can then call the logged_in() function to check if a user is currently logged in:

```
current_user.logged_in()
```

## Deployment

### Setting Up
Before you start, ensure that [Docker](https://www.docker.com/), [Ubuntu](https://ubuntu.com/), and [Visual Studio](https://visualstudio.microsoft.com/) are installed, and create an account on [Shuttle](https://www.shuttle.rs/).

Follow these steps in your terminal:

1. **Install the `cargo-shuttle` Tool:**
    ```bash
    cargo install cargo-shuttle
    ```

2. **Initialize Your Project with Axum:**
    ```bash
    cargo shuttle init --axum
    ```

3. **Add the Tera Template Engine to Your Project:**
    ```bash
    cargo add tera
    ```

4. **Load All Files in Your Shuttle Root Directory.**

### Testing

5. **Run the Project Locally for Testing:**
    ```bash
    cargo shuttle run
    ```

6. **Start the Shuttle Project:**
    ```bash
    cargo shuttle project start
    ```

7. **Deploy Your Project with Shuttle:**
    ```bash
    cargo shuttle deploy
    ```

These steps offer a concise guide to set up an Axum project, add dependencies, and deploy it using Shuttle. Ensure that you have the necessary tools and accounts in place before proceeding.


## References and Test Environment
Please ensure that you have set up the Shuttle environment. For more information, refer to the following link:

[Building an authentication system in Rust using session tokens](https://www.shuttle.rs/blog/2022/08/11/authentication-tutorial)

Special thanks to the article above for inspiration and guidance in creating this system.
use rocket::form::Form;
use rocket::response::Redirect;
use rocket::response::content::RawHtml;
use crate::state::AppState;
use rocket::State;
// admin routes moved to routes/admin.rs
use crate::auth::AuthenticatedUser;
use crate::models::access_code::AccessCode;
use rocket::serde::json::{Json, json};
use rocket::response::status::Created;
use rocket::http::Status;
use rusqlite::params;

#[derive(FromForm)]
pub struct Login {
    password: String,
}

#[get("/")]
pub fn index(state: &State<AppState>) -> RawHtml<String> {
    let is_auth = state.is_authenticated.lock().unwrap();
    
    if *is_auth {
        RawHtml(String::from(r#"
            <!DOCTYPE html>
            <html>
                <head><title>Winter Card</title></head>
                <body>
                    <h1>Welcome to Winter Card</h1>
                    <p>You are authenticated!</p>
                    <form action="/logout" method="post">
                        <button type="submit">Logout</button>
                    </form>
                </body>
            </html>
        "#))
    } else {
        RawHtml(String::from(r#"
            <!DOCTYPE html>
            <html>
                <head><title>Winter Card - Login</title></head>
                <body>
                    <h1>Please Login</h1>
                    <form action="/login" method="post">
                        <input type="password" name="password" placeholder="Enter password" required minlength="8">
                        <button type="submit">Login</button>
                    </form>
                </body>
            </html>
        "#))
    }
}

#[post("/login", data = "<login_form>")]
pub fn login(login_form: Form<Login>, state: &State<AppState>) -> Redirect {
    let conn = state.db_pool.get().expect("db connection");
    
    let result: Result<bool, _> = conn.query_row(
        "SELECT active FROM access_codes WHERE code = ?1 AND active = 1",
        [&login_form.password],
        |row| row.get(0),
    );

    if let Ok(true) = result {
        let mut is_auth = state.is_authenticated.lock().unwrap();
        *is_auth = true;
    }

    Redirect::to("/")
}

#[post("/logout")]
pub fn logout(state: &State<AppState>) -> Redirect {
    let mut auth = state.is_authenticated.lock().unwrap();
    *auth = false;
    Redirect::to("/")
}
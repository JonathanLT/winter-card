use rocket::form::Form;
use rocket::response::Redirect;
use rocket::response::content::RawHtml;
use crate::state::AppState;
use rocket::State;
use rusqlite::params;
use crate::models::access_code::AccessCode;

#[derive(FromForm)]
pub struct Login {
    password: String,
}

#[get("/")]
pub fn index(state: &State<AppState>) -> RawHtml<String> {
    let is_auth = state.is_authenticated.lock().unwrap();
    
    if *is_auth {
        RawHtml(format!(r#"
            <!DOCTYPE html>
            <html>
                <head><title>Winter Card</title></head>
                <body>
                    <h1>Hello {user_name} and Welcome to Winter Card</h1>
                    <p>You are authenticated!</p>
                    <nav>
                        <a href="/secret_santa">Secret Santa 2025</a>
                    </nav>
                    <form action="/logout" method="post">
                        <button type="submit">Logout</button>
                    </form>
                </body>
            </html>
        "#, user_name = state.current_access_code.lock().unwrap().as_ref().unwrap().name))
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

    // Récupérer le code d'accès complet
    let access_code_res = conn.query_row(
        "SELECT id, name, code, active FROM access_codes WHERE code = ?1 AND active = 1",
        params![&login_form.password],
        |row| {
            Ok(AccessCode {
                id: row.get(0)?,
                name: row.get(1)?,
                code: row.get(2)?,
                active: row.get::<_, i64>(3)? != 0,
            })
        },
    );

    if let Ok(access_code) = access_code_res {
        // marquer comme authentifié
        if let Ok(mut is_auth) = state.is_authenticated.lock() {
            *is_auth = true;
        }

        // stocker l'access code
        if let Ok(mut current_access) = state.current_access_code.lock() {
            *current_access = Some(access_code);
        }

        // associer current_user
        if let Ok(user_id) = conn.query_row(
            "SELECT id FROM users WHERE password = ?1 LIMIT 1",
            params![&login_form.password],
            |r| r.get::<_, i64>(0),
        ) {
            if let Ok(mut cur) = state.current_user.lock() {
                *cur = Some(user_id);
            }
        }
    }

    Redirect::to("/")
}

// Mettre à jour aussi la fonction logout pour réinitialiser l'access code
#[post("/logout")]
pub fn logout(state: &State<AppState>) -> Redirect {
    if let Ok(mut auth) = state.is_authenticated.lock() {
        *auth = false;
    }
    if let Ok(mut cur) = state.current_user.lock() {
        *cur = None;
    }
    if let Ok(mut access) = state.current_access_code.lock() {
        *access = None;
    }
    Redirect::to("/")
}
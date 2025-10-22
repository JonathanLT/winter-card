use rocket::form::Form;
use rocket::State;
use rocket::response::Redirect;
use rocket_dyn_templates::{Template, context};
use crate::state::AppState;
use rusqlite::params;
use crate::models::access_code::AccessCode;

#[derive(FromForm)]
pub struct LoginForm {
    password: String,
}


#[get("/")]
pub fn index(state: &State<AppState>) -> Template {
    let is_authenticated = *state.is_authenticated.lock().unwrap();
    
    if is_authenticated {
        let conn: r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager> = state.db_pool.get().expect("db connection");
        let user_id = state.current_access_code.lock().unwrap().as_ref().unwrap().id;
        let access_code_res = conn.query_row(
            "SELECT * FROM access_codes WHERE id = ?1 AND active = 1",
            params![user_id],
            |row| {
                Ok(AccessCode {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    code: row.get(2)?,
                    active: row.get::<_, i64>(3)? != 0,
                })
            },
        );

        Template::render("index", context! {
            is_authenticated: true,
            current_access_code: access_code_res.ok()
        })
        
    } else {
        Template::render("login", context! {
            is_authenticated: false,
            error: None::<String>
        })
    }
}

#[post("/login", data = "<form>")]
pub async fn login(form: Form<LoginForm>, state: &State<AppState>) -> Result<Redirect, Template> {
    let conn = state.db_pool.get().expect("db connection");

    // Récupérer le code d'accès complet
    let access_code_res = conn.query_row(
        "SELECT id, name, code, active FROM access_codes WHERE code = ?1 AND active = 1",
        params![&form.password],
        |row| {
            Ok(AccessCode {
                id: row.get(0)?,
                name: row.get(1)?,
                code: row.get(2)?,
                active: row.get::<_, i64>(3)? != 0,
            })
        },
    );

    match access_code_res {
        Ok(access_code) => {
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
                params![&form.password],
                |r| r.get::<_, i64>(0),
            ) {
                if let Ok(mut cur) = state.current_user.lock() {
                    *cur = Some(user_id);
                }
            }
            Ok(Redirect::to("/"))
        }
        Err(e) => Err(Template::render("login", context! {
            is_authenticated: false,
            error: Some(e.to_string())
        }))
    }
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
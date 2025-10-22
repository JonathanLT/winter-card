use rocket::response::content::RawHtml;
use rocket::serde::json::Json;
use rocket::response::Redirect;
use rocket::response::status::Created;
use rocket::http::Status;
use rocket::State;
use rusqlite::params;
use rocket_dyn_templates::{Template, context};
use serde_json::json;

use crate::auth::AuthenticatedUser;
use crate::state::AppState;
use crate::models::access_code::AccessCode;
use crate::models::draw::Draw;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AccessCodeWithDraw {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub active: bool,
    pub drawn: bool,
    pub receiver_id: Option<i64>,
    pub year: Option<i32>,
}
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct CreateAccessCode {
    pub name: String,
    pub code: String,
    pub active: bool,
}

#[get("/admin")]
pub fn admin_panel(_auth: AuthenticatedUser, state: &State<AppState>) -> Template {
    let current_access_name = state
        .current_access_code
        .lock()
        .unwrap()
        .as_ref()
        .map(|ac| ac.name.clone());
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

    // Redirect to home if not admin
    if user_id != 1 {
        return Template::render("index", context! {
            is_authenticated: true,
            current_access_code: access_code_res.ok()
        });
    }

    Template::render("admin", context! {
        is_authenticated: true,
        current_access_code_name: current_access_name,
        current_access_code: access_code_res.ok()
    })
}

#[get("/admin/api/codes")]
pub fn list_access_codes(_auth: AuthenticatedUser, state: &State<AppState>) -> Result<Json<Vec<AccessCodeWithDraw>>, Status> {
    let conn = state.db_pool.get().map_err(|_| Status::InternalServerError)?;
    let mut stmt_access_codes = conn.prepare("SELECT id, name, code, active FROM access_codes")
        .map_err(|_| Status::InternalServerError)?;

    let codes_iter = stmt_access_codes.query_map([], |row| {
        let id: i64 = row.get(0)?;
        let name: String = row.get(1)?;
        let code: String = row.get(2)?;
        // read stored integer and convert to bool
        let active_int: i64 = row.get(3)?;
        Ok(AccessCode {
            id: id,
            name,
            code,
            active: active_int != 0,
        })
    }).map_err(|_| Status::InternalServerError)?;

    let mut stmt_draws = conn.prepare("SELECT id, giver_id, receiver_id, year, created_at FROM draws")
        .map_err(|_| Status::InternalServerError)?;

    let draws_iter = stmt_draws.query_map([], |row| {
        let id: i64 = row.get(0)?;
        let giver_id: i64 = row.get(1)?;
        let receiver_id: i64 = row.get(2)?;
        let year: i32 = row.get(3)?;
        let created_at: String = row.get(4)?;
        Ok(Draw {
            id: id,
            giver_id: giver_id,
            receiver_id: receiver_id,
            year: year,
            created_at: created_at,
        })
    }).map_err(|_| Status::InternalServerError)?;


    let codes: Vec<AccessCode> = codes_iter.filter_map(Result::ok).collect();
    let draws: Vec<Draw> = draws_iter.filter_map(Result::ok).collect();

    let codes_with_draws: Vec<AccessCodeWithDraw> = codes.into_iter().map(|code| {
        let draw_opt = draws.iter().find(|draw| draw.giver_id == code.id);
        if let Some(draw) = draw_opt {
            AccessCodeWithDraw {
                id: code.id,
                name: code.name,
                code: code.code,
                active: code.active,
                drawn: true,
                receiver_id: Some(draw.receiver_id),
                year: Some(draw.year),
            }
        } else {
            AccessCodeWithDraw {
                id: code.id,
                name: code.name,
                code: code.code,
                active: code.active,
                drawn: false,
                receiver_id: None,
                year: None,
            }
        }
    }).collect();

    Ok(Json(codes_with_draws))
}

#[post("/admin/api/codes", data = "<code>")]
pub fn create_access_code(_auth: AuthenticatedUser, code: Json<CreateAccessCode>, state: &State<AppState>) -> Result<Created<Json<AccessCode>>, Status> {
    let conn = state.db_pool.get().map_err(|_| Status::InternalServerError)?;
    
    conn.execute(
        "INSERT INTO access_codes (name, code, active) VALUES (?1, ?2, ?3)",
        params![code.name, code.code, if code.active { 1 } else { 0 }],
    ).map_err(|_| Status::InternalServerError)?;

    let id = conn.last_insert_rowid();
    let created_code = AccessCode {
        id: id,
        name: code.name.clone(),
        code: code.code.clone(),
        active: code.active,
    };

    Ok(Created::new("/admin/api/codes").body(Json(created_code)))
}

#[patch("/admin/api/codes/<id>", data = "<code>")]
pub fn update_access_code(
    _auth: AuthenticatedUser,
    id: i64,
    code: Json<AccessCode>,
    state: &State<AppState>
) -> Result<Json<serde_json::Value>, Status> {
    let conn = state.db_pool.get().map_err(|_| Status::InternalServerError)?;
    
    let rows_affected = conn.execute(
        "UPDATE access_codes SET name = ?1, code = ?2, active = ?3 WHERE id = ?4",
        params![
            code.name,
            code.code,
            if code.active { 1 } else { 0 },
            id
        ],
    ).map_err(|_| Status::InternalServerError)?;

    if rows_affected == 0 {
        return Err(Status::NotFound);
    }

    Ok(Json(json!({
        "status": "success",
        "message": "Code mis à jour avec succès",
        "toast": {
            "type": "success",
            "message": "Code mis à jour avec succès"
        }
    })))
}

#[delete("/admin/api/codes/<id>")]
pub fn delete_access_code(_auth: AuthenticatedUser, id: i64, state: &State<AppState>) -> Result<Status, Status> {
    let conn = state.db_pool.get().map_err(|_| Status::InternalServerError)?;
    let rows_affected = conn.execute(
        "DELETE FROM access_codes WHERE id = ?1",
        params![id],
    ).map_err(|_| Status::InternalServerError)?;
    if rows_affected == 0 {
        return Err(Status::NotFound);
    }
    Ok(Status::NoContent)
}

// Test code
#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use rocket::local::blocking::Client;
    use rocket::http::{Status, ContentType};
    use crate::db::{init_pool, init_db};
    use std::sync::Mutex;
    use rocket::serde::json::serde_json;

    fn setup_rocket() -> rocket::Rocket<rocket::Build> {
        let pool = init_pool(":memory:");
        init_db(&pool);
        let state = AppState {
            db_pool: pool,
            is_authenticated: Mutex::new(true),
            current_user: Mutex::new(None),
            current_access_code: Mutex::new(None),
        };

        rocket::build()
            .manage(state)
            .mount("/", routes![
                admin_panel,
                list_access_codes,
                create_access_code,
                update_access_code,
                delete_access_code,
            ])
            .attach(Template::fairing())

    }

    // Test admin route create_access_code
    #[test]
    fn test_create_access_code() {
        let rocket = setup_rocket();
        let client = Client::tracked(rocket).expect("valid rocket instance");
        let new_code = CreateAccessCode {
            name: "Test Code".to_string(),
            code: "TESTCODE".to_string(),
            active: true,
        };
        let response = client.post("/admin/api/codes")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&new_code).unwrap())
            .dispatch();
        assert_eq!(response.status(), Status::Created);
        let created_code: AccessCode = response.into_json().expect("valid json");
        assert_eq!(created_code.name, "Test Code");
        assert_eq!(created_code.code, "TESTCODE");
        assert_eq!(created_code.active, true);
        assert!(created_code.id > 0);
    }

    // Test admin route list_access_codes
    #[test]
    fn test_list_access_codes() {
        let rocket = setup_rocket();
        let client = Client::tracked(rocket).expect("valid rocket instance");
        let new_code = CreateAccessCode {
            name: "Test Code".to_string(),
            code: "TESTCODE".to_string(),
            active: true,
        };
        let _ = client.post("/admin/api/codes")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&new_code).unwrap())
            .dispatch();
        let response = client.get("/admin/api/codes").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let codes: Vec<AccessCodeWithDraw> = response.into_json().expect("valid json");
        assert!(codes.iter().any(|c| c.code == "TESTCODE"));
    }

    // Test admin route update_access_code
    #[test]
    fn test_update_access_code() {
        let rocket = setup_rocket();
        let client = Client::tracked(rocket).expect("valid rocket instance");
        
        // First create a code
        let new_code = CreateAccessCode {
            name: "Test Code".to_string(),
            code: "TESTCODE".to_string(),
            active: true,
        };
        let response = client.post("/admin/api/codes")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&new_code).unwrap())
            .dispatch();
        let created_code: AccessCode = response.into_json().expect("valid json");
        
        // Then update it
        let update_code = AccessCode {
            id: created_code.id,
            name: "Updated Code".to_string(),
            code: "UPDATEDCODE".to_string(),
            active: false,
        };
        
        let response = client.patch(format!("/admin/api/codes/{}", created_code.id))
            .header(ContentType::JSON)
            .body(serde_json::to_string(&update_code).unwrap())
            .dispatch();
            
        assert_eq!(response.status(), Status::Ok);
        
        // Verify the update
        let response = client.get("/admin/api/codes").dispatch();
        let codes: Vec<AccessCodeWithDraw> = response.into_json().expect("valid json");
        let updated_code = codes.iter()
            .find(|c| c.id == created_code.id)
            .expect("code exists");
            
        assert_eq!(updated_code.name, "Updated Code");
        assert_eq!(updated_code.code, "UPDATEDCODE");
        assert_eq!(updated_code.active, false);
    }

    // Test admin route delete_access_code
    #[test]
    fn test_delete_access_code() {
        let rocket = setup_rocket();
        let client = Client::tracked(rocket).expect("valid rocket instance");
        let new_code = CreateAccessCode {
            name: "Test Code".to_string(),
            code: "TESTCODE".to_string(),
            active: true,
        };
        let response = client.post("/admin/api/codes")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&new_code).unwrap())
            .dispatch();
        let created_code: AccessCode = response.into_json().expect("valid json");
        let delete_resp = client.delete(format!("/admin/api/codes/{}", created_code.id))
            .dispatch();
        assert_eq!(delete_resp.status(), Status::NoContent);
    }
}
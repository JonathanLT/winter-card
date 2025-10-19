use rocket::response::content::RawHtml;
use rocket::serde::json::Json;
use rocket::response::status::Created;
use rocket::http::Status;
use rocket::State;
use rusqlite::params;

use crate::auth::AuthenticatedUser;
use crate::state::AppState;
use crate::models::access_code::AccessCode;

#[get("/admin")]
pub fn admin_panel(_auth: AuthenticatedUser) -> RawHtml<&'static str> {
    RawHtml(r#"
        <!DOCTYPE html>
        <html>
            <head>
                <title>Admin Panel - Access Codes</title>
                <style>
                    table { border-collapse: collapse; width: 100%; margin-top: 20px; }
                    th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
                    th { background-color: #f2f2f2; }
                    .inactive { color: red; }
                    .form-container { margin: 20px 0; padding: 20px; background-color: #f9f9f9; }
                </style>
                <script>
                    async function loadAccessCodes() {
                        const response = await fetch('/admin/api/codes');
                        if (!response.ok) {
                            document.getElementById('codeList').innerHTML = '<tr><td colspan=\"4\">Erreur de chargement</td></tr>';
                            return;
                        }
                        const codes = await response.json();
                        const codeList = document.getElementById('codeList');
                        codeList.innerHTML = codes.map(code => `
                            <tr class="${code.active ? '' : 'inactive'}">
                                <td>${code.id}</td>
                                <td>${code.code}</td>
                                <td>${code.active ? 'Active' : 'Inactive'}</td>
                                <td>
                                    <button onclick="toggleCode(${code.id})">Toggle</button>
                                    <button onclick="deleteCode(${code.id})">Delete</button>
                                </td>
                            </tr>
                        `).join('');
                    }

                    async function createCode(event) {
                        event.preventDefault();
                        const form = event.target;
                        const response = await fetch('/admin/api/codes', {
                            method: 'POST',
                            headers: {
                                'Content-Type': 'application/json'
                            },
                            body: JSON.stringify({
                                code: form.code.value,
                                active: form.active.checked
                            })
                        });

                        if (response.ok) {
                            form.reset();
                            loadAccessCodes();
                        } else {
                            alert('Error creating access code');
                        }
                    }
                    async function deleteCode(id) {
                        const response = await fetch('/admin/api/codes/' + id, {
                            method: 'DELETE',
                        });

                        if (response.ok) {
                            loadAccessCodes();
                        } else {
                            alert('Error deleting access code');
                        }
                    }
                    async function toggleCode(id) {
                        const response = await fetch('/admin/api/codes/' + id, {
                            method: 'PATCH',
                        });

                        if (response.ok) {
                            loadAccessCodes();
                        } else {
                            alert('Error toggling access code');
                        }
                    }

                    window.onload = loadAccessCodes;
                </script>
            </head>
            <body>
                <h1>Access Codes Management</h1>

                <div class="form-container">
                    <h2>Create New Access Code</h2>
                    <form onsubmit="createCode(event)">
                        <input type="text" name="code" 
                               pattern="^[A-Za-z0-9]{8,}$" 
                               title="Minimum 8 characters, letters and numbers only" 
                               required 
                               placeholder="Enter access code">
                        <label>
                            <input type="checkbox" name="active" checked> Active
                        </label>
                        <button type="submit">Create Code</button>
                    </form>
                </div>

                <table>
                    <thead>
                        <tr>
                            <th>ID</th>
                            <th>Code</th>
                            <th>Status</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody id="codeList">
                        <tr>
                            <td colspan="3">Loading...</td>
                        </tr>
                    </tbody>
                </table>

                <p><a href="/">Back to Home</a></p>
            </body>
        </html>
    "#)
}

#[get("/admin/api/codes")]
pub fn list_access_codes(_auth: AuthenticatedUser, state: &State<AppState>) -> Result<Json<Vec<AccessCode>>, Status> {
    let conn = state.db_pool.get().map_err(|_| Status::InternalServerError)?;
    let mut stmt = conn.prepare("SELECT id, code, active FROM access_codes")
        .map_err(|_| Status::InternalServerError)?;

    let codes_iter = stmt.query_map([], |row| {
        let id: i64 = row.get(0)?;
        let code: String = row.get(1)?;
        // read stored integer and convert to bool
        let active_int: i64 = row.get(2)?;
        Ok(AccessCode {
            id: Some(id),
            code,
            active: active_int != 0,
        })
    }).map_err(|_| Status::InternalServerError)?;

    let codes: Vec<AccessCode> = codes_iter.filter_map(Result::ok).collect();
    Ok(Json(codes))
}

#[post("/admin/api/codes", data = "<code>")]
pub fn create_access_code(_auth: AuthenticatedUser, code: Json<AccessCode>, state: &State<AppState>) -> Result<Created<Json<AccessCode>>, Status> {
    let conn = state.db_pool.get().map_err(|_| Status::InternalServerError)?;
    
    conn.execute(
        "INSERT INTO access_codes (code, active) VALUES (?1, ?2)",
        params![code.code, if code.active { 1 } else { 0 }],
    ).map_err(|_| Status::InternalServerError)?;

    let id = conn.last_insert_rowid();
    let created_code = AccessCode {
        id: Some(id),
        code: code.code.clone(),
        active: code.active,
    };

    Ok(Created::new("/admin/api/codes").body(Json(created_code)))
}

#[patch("/admin/api/codes/<id>")]
pub fn toggle_access_code(_auth: AuthenticatedUser, id: i64, state: &State<AppState>) -> Result<Status, Status> {
    let conn = state.db_pool.get().map_err(|_| Status::InternalServerError)?;
    
    let rows_affected = conn.execute(
        // use CASE to be explicit and portable
        "UPDATE access_codes SET active = CASE WHEN active = 1 THEN 0 ELSE 1 END WHERE id = ?1",
        params![id],
    ).map_err(|_| Status::InternalServerError)?;

    if rows_affected == 0 {
        return Err(Status::NotFound);
    }

    Ok(Status::NoContent)
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
        };

        rocket::build()
            .manage(state)
            .mount("/", routes![
                admin_panel,
                list_access_codes,
                create_access_code,
                toggle_access_code,
                delete_access_code,
            ])
    }

    // Test admin route create_access_code
    #[test]
    fn test_create_access_code() {
        let rocket = setup_rocket();
        let client = Client::tracked(rocket).expect("valid rocket instance");
        let new_code = AccessCode {
            id: None,
            code: "TESTCODE".to_string(),
            active: true,
        };
        let response = client.post("/admin/api/codes")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&new_code).unwrap())
            .dispatch();
        assert_eq!(response.status(), Status::Created);
        let created_code: AccessCode = response.into_json().expect("valid json");
        assert_eq!(created_code.code, "TESTCODE");
        assert_eq!(created_code.active, true);
        assert!(created_code.id.is_some());
    }

    // Test admin route list_access_codes
    #[test]
    fn test_list_access_codes() {
        let rocket = setup_rocket();
        let client = Client::tracked(rocket).expect("valid rocket instance");
        let new_code = AccessCode {
            id: None,
            code: "TESTCODE".to_string(),
            active: true,
        };
        let response = client.post("/admin/api/codes")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&new_code).unwrap())
            .dispatch();
        assert_eq!(response.status(), Status::Created);
        let created_code: AccessCode = response.into_json().expect("valid json");
        assert_eq!(created_code.code, "TESTCODE");
        assert_eq!(created_code.active, true);
        assert!(created_code.id.is_some());
        let response = client.get("/admin/api/codes").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let codes: Vec<AccessCode> = response.into_json().expect("valid json");
        assert!(codes.iter().any(|c| c.code == "TESTCODE"));
    }

    // Test admin route toggle_access_code
    #[test]
    fn test_toggle_access_code() {
        let rocket = setup_rocket();
        let client = Client::tracked(rocket).expect("valid rocket instance");
        let new_code = AccessCode {
            id: None,
            code: "TESTCODE".to_string(),
            active: true,
        };
        let response = client.post("/admin/api/codes")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&new_code).unwrap())
            .dispatch();
        assert_eq!(response.status(), Status::Created);
        let created_code: AccessCode = response.into_json().expect("valid json");
        assert_eq!(created_code.code, "TESTCODE");
        assert_eq!(created_code.active, true);
        assert!(created_code.id.is_some());
        let response = client.get("/admin/api/codes").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let codes: Vec<AccessCode> = response.into_json().expect("valid json");
        assert!(codes.iter().any(|c| c.code == "TESTCODE"));
        let response = client.patch(format!("/admin/api/codes/{}", created_code.id.unwrap())).dispatch();
        assert_eq!(response.status(), Status::NoContent);
        let response = client.get("/admin/api/codes").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let codes: Vec<AccessCode> = response.into_json().expect("valid json");
        let toggled_code = codes.iter().find(|c| c.code == "TESTCODE").expect("code exists");
        assert_eq!(toggled_code.active, false);
    }

    // Test admin route delete_access_code
    #[test]
    fn test_delete_access_code() {
        let rocket = setup_rocket();
        let client = Client::tracked(rocket).expect("valid rocket instance");
        let new_code = AccessCode {
            id: None,
            code: "TESTCODE".to_string(),
            active: true,
        };
        let response = client.post("/admin/api/codes")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&new_code).unwrap())
            .dispatch();
        assert_eq!(response.status(), Status::Created);
        let created_code: AccessCode = response.into_json().expect("valid json");
        assert_eq!(created_code.code, "TESTCODE");
        assert_eq!(created_code.active, true);
        assert!(created_code.id.is_some());
        let response = client.get("/admin/api/codes").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let codes: Vec<AccessCode> = response.into_json().expect("valid json");
        assert!(codes.iter().any(|c| c.code == "TESTCODE"));
        let response = client.delete(format!("/admin/api/codes/{}", created_code.id.unwrap())).dispatch();
        assert_eq!(response.status(), Status::NoContent);
        let response = client.get("/admin/api/codes").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let codes: Vec<AccessCode> = response.into_json().expect("valid json");
        assert!(!codes.iter().any(|c| c.code == "TESTCODE"));
    }
}
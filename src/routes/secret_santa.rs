use rocket::response::content::RawHtml;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::http::Status;
use rocket::State;
use rusqlite::params;
use chrono::Datelike;

use crate::auth::AuthenticatedUser;
use crate::state::AppState;
use rocket_dyn_templates::{Template, context};

#[get("/secret_santa")]
pub fn secret_santa(_auth: AuthenticatedUser, state: &State<AppState>) -> Template {
    // récupérer id connecté (optionnel)
    let user_id_val = state.current_access_code.lock().unwrap().as_ref().unwrap().id;
    println!("User ID connecté: {}", user_id_val);

    // vérifier si l'utilisateur a déjà tiré au sort cette année
    let conn = state.db_pool.get().expect("db connection");
    let current_year = chrono::Utc::now().year();
    let already_drawn = conn.query_row(
        "SELECT COUNT(*) FROM draws WHERE giver_id = ?1 AND year = ?2",
        params![user_id_val, current_year],
        |row| row.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    let draw_button_state = if already_drawn { "disabled" } else { "" };


    // Récupérer le nom du destinataire assigné (si déjà tiré)
    let receiver_name = if already_drawn {
        conn.query_row(
            "
            SELECT access_codes.name
            FROM draws
            INNER JOIN access_codes ON draws.receiver_id = access_codes.id
            WHERE draws.giver_id = ?1 AND draws.year = ?2
            ",
            params![user_id_val, current_year],
            |row| row.get::<_, String>(0),
        ).unwrap_or("Inconnu".to_string())
    } else {
        "Inconnu".to_string()
    };

    // Render the `secret_santa` template
    Template::render("secret_santa", context! {
        is_authenticated: true,
        user_id: user_id_val,
        draw_button_state,
        hidden_draw: if already_drawn { "" } else { "hidden" },
        receiver_name,
    })
}

#[derive(Deserialize)]
pub struct DrawRequest {
    user_id: i64,
}

#[derive(Serialize)]
pub struct DrawResult {
    assigned_id: i64,
    assigned_name: String,
}

#[post("/secret_santa/api/draw", data = "<req>")]
pub fn perform_draw(_auth: AuthenticatedUser, req: Json<DrawRequest>, state: &State<AppState>) -> Result<Json<DrawResult>, Status> {
    let conn = state.db_pool.get().map_err(|_| Status::InternalServerError)?;
    let current_year = chrono::Utc::now().year();
    
    // choisir une personne aléatoire autre que le demandeur
    let row_res = conn.query_row(
        "
        SELECT access_codes.id, name
        FROM access_codes 
        WHERE 
            access_codes.id != ?1
            AND access_codes.active == 1
            AND access_codes.id NOT IN (SELECT receiver_id FROM draws WHERE year == ?2)
        ORDER BY RANDOM()
        LIMIT 1;
        ",
        params![req.user_id, current_year],
        |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)),
    );

    // insérer le tirage dans la table draws pour l'année en cours
    if let Ok((assigned_id, _)) = &row_res {
        let insert_res = conn.execute(
            "INSERT INTO draws (giver_id, receiver_id, year) VALUES (?1, ?2, ?3)",
            params![req.user_id, assigned_id, current_year],
        );
        if insert_res.is_err() {
            return Err(Status::InternalServerError);
        }
    }

    match row_res {
        Ok((assigned_id, assigned_name)) => Ok(Json(DrawResult { assigned_id, assigned_name })),
        Err(rusqlite::Error::QueryReturnedNoRows) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}
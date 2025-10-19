use rocket::response::content::RawHtml;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::http::Status;
use rocket::State;
use rusqlite::params;
use chrono::Datelike;

use crate::auth::AuthenticatedUser;
use crate::state::AppState;

#[get("/secret_santa")]
pub fn secret_santa(_auth: AuthenticatedUser, state: &State<AppState>) -> RawHtml<String> {
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

    RawHtml(format!(r#"
        <!DOCTYPE html>
        <html lang="fr">
        <head>
            <meta charset="utf-8" />
            <meta name="viewport" content="width=device-width,initial-scale=1" />
            <title>Secret Santa - Winter Card</title>
            <style>
                body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial; background:#f7fafc; color:#222; margin:0; padding:24px; }}
                .container {{ max-width:900px; margin:0 auto; background:#fff; padding:28px; border-radius:10px; box-shadow:0 6px 18px rgba(0,0,0,0.06); }}
                h1 {{ color:#b22222; margin-top:0; }}
                .nav {{ margin-bottom:16px; }}
                .nav a {{ margin-right:12px; color:#b22222; text-decoration:none; }}
                .note {{ margin-top:18px; background:#fff7e6; padding:12px; border-left:4px solid #ffd580; border-radius:4px; }}
                button {{ background:#b22222; color:#fff; border:none; padding:8px 12px; border-radius:6px; cursor:pointer; }}
                button:disabled {{ opacity:0.5; cursor:default; }}
            </style>
        </head>
        <body>
            <div class="container">
                <div class="nav">
                    <a href="/">Accueil</a>
                    <a href="/admin">Admin</a>
                </div>

                <h1>🎅 Secret Santa</h1>

                <p>
                    Le <strong>Secret Santa</strong> (ou « Père Noël secret » en français) est un jeu ou une tradition populaire pendant la période de Noël,
                    souvent organisé entre amis, collègues ou membres d’une famille. Voici le <strong>principe</strong> :
                </p>

                <ol>
                    <li><strong>🎁 Tirage au sort anonyme</strong>
                        <ul>
                            <li>Chaque participant tire au hasard le nom d’une autre personne du groupe.</li>
                            <li>Il devient alors le « Secret Santa » (le Père Noël secret) de cette personne.</li>
                            <li>L’identité de celui qui offre le cadeau reste <strong>secrète</strong> jusqu’à la fin.</li>
                        </ul>
                    </li>
                    <li><strong>💡 Budget fixé à l’avance</strong>
                        <ul><li>Le groupe s’accorde sur un <strong>montant maximum</strong> pour que les cadeaux soient équitables.</li></ul>
                    </li>
                    <li><strong>🎀 Achat et échange des cadeaux</strong>
                        <ul><li>Chacun achète un petit cadeau pour la personne tirée au sort.</li></ul>
                    </li>
                    <li><strong>🤫 Option : garder le secret ou le révéler</strong>
                        <ul><li>Parfois on garde le secret, parfois on révèle à la fin.</li></ul>
                    </li>
                </ol>

                <div class="note">
                    👉 Le but principal est de <strong>partager un moment amusant et chaleureux</strong> sans que chacun ait à acheter pour tout le monde.
                </div>

                <div style="margin-top:20px;">
                    <!-- bouton de tirage : contient l'id utilisateur connecté -->
                    <button id="drawBtn" {draw_button_state} data-user-id="{user_id}">Tirer au sort</button>
                    <span id="drawResult" style="margin-left:12px;"></span>
                </div>
            </div>

            <script>
                async function draw() {{
                    const btn = document.getElementById('drawBtn');
                    const userId = btn.getAttribute('data-user-id');
                    if (!userId) {{
                        alert('ID utilisateur non disponible.');
                        return;
                    }}
                    console.log('ID utilisateur :', Number(userId));
                    btn.disabled = true;
                    const res = await fetch('/secret_santa/api/draw', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ user_id: Number(userId) }})
                    }});
                    if (!res.ok) {{
                        const text = await res.text();
                        document.getElementById('drawResult').textContent = 'Erreur: ' + res.status + ' ' + text;
                        return;
                    }}
                    const json = await res.json();
                    document.getElementById('drawResult').textContent = 'Vous devez offrir à : ' + json.assigned_name + ' (id=' + json.assigned_id + ')';
                }}

                document.getElementById('drawBtn').addEventListener('click', draw);
            </script>
        </body>
        </html>
    "#, user_id = user_id_val))
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
use rocket::Route;

pub mod index;
pub mod admin;
pub mod secret_santa;

pub fn routes() -> Vec<Route> {
    routes![
        // page publique / privée
        index::index,
        index::login,
        index::logout,

        // admin (page + API)
        admin::admin_panel,
        admin::list_access_codes,
        admin::create_access_code,
        admin::update_access_code,
        admin::delete_access_code,

        // secret santa
        secret_santa::secret_santa,
        secret_santa::perform_draw, // nouvelle route pour le tirage
    ]
}
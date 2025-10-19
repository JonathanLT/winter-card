use rocket::Route;

pub mod index;
pub mod admin;

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
        admin::toggle_access_code,
        admin::delete_access_code,
    ]
}
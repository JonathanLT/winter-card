use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;
use crate::state::AppState;

pub struct AuthenticatedUser;

#[derive(Debug)]
pub enum AuthError {
    NotAuthenticated,
    MissingState,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = AuthError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let state = match request.rocket().state::<AppState>() {
            Some(state) => state,
            None => return Outcome::Error((Status::InternalServerError, AuthError::MissingState)),
        };

        let is_auth = match state.is_authenticated.lock() {
            Ok(guard) => *guard,
            Err(_) => return Outcome::Error((Status::InternalServerError, AuthError::NotAuthenticated)),
        };

        if is_auth {
            Outcome::Success(AuthenticatedUser)
        } else {
            Outcome::Forward(Status::Unauthorized)
        }
    }
}
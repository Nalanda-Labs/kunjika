use crate::middlewares::auth::AuthorizationService;
use crate::state::AppState;

pub async fn verify_profile_user(uid: i64, auth: &AuthorizationService) -> bool {
    if auth.claims.id != uid {
        return false;
    }

    return true;
}
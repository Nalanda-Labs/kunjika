use crate::middlewares::auth::AuthorizationService;

pub async fn verify_profile_user(uid: i64, auth: &AuthorizationService) -> bool {
    if auth.user.id != uid {
        return false;
    }

    return true;
}

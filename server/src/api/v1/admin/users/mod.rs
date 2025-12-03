use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};

use crate::db::dto::Pagination;
use crate::db::models::{CreateUserDto, UpdateUserDto, UserModel};
use crate::db::repositories::{TUserRepository, UserRepository};
use crate::middleware::auth::AuthenticatedUser;
use crate::AppState;

// DTOs
#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_limit")]
    pub limit: u64,
    pub search: Option<String>,
    pub role: Option<String>,
    pub disabled: Option<bool>,
    pub locked_out: Option<bool>,
}

fn default_page() -> u64 {
    1
}

fn default_limit() -> u64 {
    20
}

#[derive(Debug, Serialize)]
pub struct AdminUserResponse {
    pub id: String,
    pub email: String,
    pub email_verified: Option<bool>,
    pub normalized_email: String,
    pub name: Option<String>,
    pub username: Option<String>,
    pub normalized_username: Option<String>,
    pub avatar_url: Option<String>,
    pub provider: String,
    pub disabled: bool,
    pub locked_out: bool,
    pub lockout_end: Option<chrono::DateTime<chrono::Utc>>,
    pub access_failed_count: i32,
    pub last_login_at: Option<chrono::DateTime<chrono::Utc>>,
    pub login_count: i32,
    pub roles: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub id: String,
    pub email: String,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub provider: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LockUserRequest {
    pub duration_minutes: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct AddRoleRequest {
    pub role_name: String,
    pub assigned_by: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListUsersResponse {
    pub data: Vec<AdminUserResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Serialize)]
pub struct UserStatsResponse {
    pub login_count: i32,
    pub last_login_at: Option<chrono::DateTime<chrono::Utc>>,
    pub access_failed_count: i32,
    pub locked_out: bool,
    pub lockout_end: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<UserModel> for AdminUserResponse {
    fn from(user: UserModel) -> Self {
        // Note: We'll need to fetch roles separately
        let normalized_email = user.email.to_lowercase();
        let normalized_username = user.username.as_ref().map(|u| u.to_lowercase());

        AdminUserResponse {
            id: user.id,
            email: user.email,
            email_verified: user.email_verified,
            normalized_email, // TODO: Get from DB
            name: user.name,
            username: user.username.clone(),
            normalized_username,
            avatar_url: user.avatar_url,
            provider: user.provider,
            disabled: false,        // TODO: Get from DB
            locked_out: false,      // TODO: Get from DB
            lockout_end: None,      // TODO: Get from DB
            access_failed_count: 0, // TODO: Get from DB
            last_login_at: None,    // TODO: Get from DB
            login_count: 0,         // TODO: Get from DB
            roles: vec![],          // Will be populated separately
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

// GET /api/v1/admin/users
pub async fn list_users(
    State(state): State<AppState>,
    Query(params): Query<ListUsersQuery>,
    _user: AuthenticatedUser,
) -> Result<Json<ListUsersResponse>, StatusCode> {
    let user_repo = UserRepository::new(state.db.clone());

    let pagination = Pagination {
        page: params.page,
        page_size: params.limit,
    };

    // TODO: Implement filtering by search, role, disabled, locked_out
    let result = user_repo
        .list(None, Some(pagination))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Fetch roles for each user
    let mut users_with_roles = Vec::new();
    for user in result.data {
        let roles = user_repo.get_user_roles(&user.id).await.unwrap_or_default();

        // All fields are now in the User model

        users_with_roles.push(AdminUserResponse {
            id: user.id,
            email: user.email,
            email_verified: user.email_verified,
            normalized_email: user.normalized_email.clone(),
            name: user.name,
            username: user.username,
            normalized_username: user.normalized_username.clone(),
            avatar_url: user.avatar_url,
            provider: user.provider,
            disabled: user.disabled,
            locked_out: user.locked_out,
            lockout_end: user.lockout_end,
            access_failed_count: user.access_failed_count,
            last_login_at: user.last_login_at,
            login_count: user.login_count,
            roles,
            created_at: user.created_at,
            updated_at: user.updated_at,
        });
    }

    Ok(Json(ListUsersResponse {
        data: users_with_roles,
        total: result.total,
        page: params.page,
        page_size: params.limit,
    }))
}

// GET /api/v1/admin/users/{id}
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    _user: AuthenticatedUser,
) -> Result<Json<AdminUserResponse>, StatusCode> {
    let user_repo = UserRepository::new(state.db.clone());

    let user = user_repo
        .get(id.clone())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let roles = user_repo.get_user_roles(&user.id).await.unwrap_or_default();

    Ok(Json(AdminUserResponse {
        id: user.id,
        email: user.email,
        email_verified: user.email_verified,
        normalized_email: user.normalized_email.clone(),
        name: user.name,
        username: user.username,
        normalized_username: user.normalized_username.clone(),
        avatar_url: user.avatar_url,
        provider: user.provider,
        disabled: user.disabled,
        locked_out: user.locked_out,
        lockout_end: user.lockout_end,
        access_failed_count: user.access_failed_count,
        last_login_at: user.last_login_at,
        login_count: user.login_count,
        roles,
        created_at: user.created_at,
        updated_at: user.updated_at,
    }))
}

// POST /api/v1/admin/users
pub async fn create_user(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<AdminUserResponse>, StatusCode> {
    let user_repo = UserRepository::new(state.db.clone());

    let create_dto = CreateUserDto {
        id: req.id,
        email: req.email,
        email_verified: req.email_verified,
        name: req.name,
        username: req.username,
        avatar_url: req.avatar_url,
        provider: req.provider,
    };

    // Convert DTO to NewUser, then to UserModel
    let new_user: crate::db::models::NewUser = create_dto.into();
    let user_model = UserModel {
        id: new_user.id.clone(),
        email: new_user.email.clone(),
        normalized_email: new_user.normalized_email.clone(),
        email_verified: new_user.email_verified,
        name: new_user.name.clone(),
        username: new_user.username.clone(),
        normalized_username: new_user.normalized_username.clone(),
        avatar_url: new_user.avatar_url.clone(),
        provider: new_user.provider.clone(),
        role: new_user.role.clone(),
        password_hash: None,
        two_factor_enabled: None,
        totp_secret: None,
        preferences: new_user.preferences.clone(),
        terms_accepted: None,
        terms_accepted_at: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        disabled: new_user.disabled,
        locked_out: new_user.locked_out,
        lockout_end: None,
        access_failed_count: new_user.access_failed_count,
        password_changed_at: None,
        last_login_at: None,
        login_count: new_user.login_count,
    };

    let user = user_repo
        .create(user_model)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let roles = user_repo.get_user_roles(&user.id).await.unwrap_or_default();

    Ok(Json(AdminUserResponse {
        id: user.id,
        email: user.email,
        email_verified: user.email_verified,
        normalized_email: user.normalized_email.clone(),
        name: user.name,
        username: user.username,
        normalized_username: user.normalized_username.clone(),
        avatar_url: user.avatar_url,
        provider: user.provider,
        disabled: user.disabled,
        locked_out: user.locked_out,
        lockout_end: user.lockout_end,
        access_failed_count: user.access_failed_count,
        last_login_at: user.last_login_at,
        login_count: user.login_count,
        roles,
        created_at: user.created_at,
        updated_at: user.updated_at,
    }))
}

// PUT /api/v1/admin/users/{id}
pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    _user: AuthenticatedUser,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<AdminUserResponse>, StatusCode> {
    let user_repo = UserRepository::new(state.db.clone());

    let update_dto = UpdateUserDto {
        name: req.name,
        username: req.username,
        avatar_url: req.avatar_url,
        preferences: None,
    };

    let user = user_repo
        .update(id.clone(), update_dto)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let roles = user_repo.get_user_roles(&user.id).await.unwrap_or_default();

    Ok(Json(AdminUserResponse {
        id: user.id,
        email: user.email,
        email_verified: user.email_verified,
        normalized_email: user.normalized_email.clone(),
        name: user.name,
        username: user.username,
        normalized_username: user.normalized_username.clone(),
        avatar_url: user.avatar_url,
        provider: user.provider,
        disabled: user.disabled,
        locked_out: user.locked_out,
        lockout_end: user.lockout_end,
        access_failed_count: user.access_failed_count,
        last_login_at: user.last_login_at,
        login_count: user.login_count,
        roles,
        created_at: user.created_at,
        updated_at: user.updated_at,
    }))
}

// DELETE /api/v1/admin/users/{id}
pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    _user: AuthenticatedUser,
) -> Result<StatusCode, StatusCode> {
    let user_repo = UserRepository::new(state.db.clone());

    user_repo
        .delete(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

// POST /api/v1/admin/users/{id}/enable
pub async fn enable_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    _user: AuthenticatedUser,
) -> Result<Json<AdminUserResponse>, StatusCode> {
    let user_repo = UserRepository::new(state.db.clone());

    user_repo
        .enable_user(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    get_user(State(state), Path(id), _user).await
}

// POST /api/v1/admin/users/{id}/disable
pub async fn disable_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    _user: AuthenticatedUser,
) -> Result<Json<AdminUserResponse>, StatusCode> {
    let user_repo = UserRepository::new(state.db.clone());

    user_repo
        .disable_user(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    get_user(State(state), Path(id), _user).await
}

// POST /api/v1/admin/users/{id}/lock
pub async fn lock_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    _user: AuthenticatedUser,
    Json(req): Json<LockUserRequest>,
) -> Result<Json<AdminUserResponse>, StatusCode> {
    let user_repo = UserRepository::new(state.db.clone());

    let duration = req.duration_minutes.unwrap_or(15);
    user_repo
        .lock_user(&id, duration)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    get_user(State(state), Path(id), _user).await
}

// POST /api/v1/admin/users/{id}/unlock
pub async fn unlock_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    _user: AuthenticatedUser,
) -> Result<Json<AdminUserResponse>, StatusCode> {
    let user_repo = UserRepository::new(state.db.clone());

    user_repo
        .unlock_user(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    get_user(State(state), Path(id), _user).await
}

// GET /api/v1/admin/users/{id}/roles
pub async fn get_user_roles(
    State(state): State<AppState>,
    Path(id): Path<String>,
    _user: AuthenticatedUser,
) -> Result<Json<Vec<String>>, StatusCode> {
    let user_repo = UserRepository::new(state.db.clone());

    let roles = user_repo
        .get_user_roles(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(roles))
}

// POST /api/v1/admin/users/{id}/roles
pub async fn add_user_role(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: AuthenticatedUser,
    Json(req): Json<AddRoleRequest>,
) -> Result<Json<Vec<String>>, StatusCode> {
    let user_repo = UserRepository::new(state.db.clone());

    user_repo
        .add_role(
            &id,
            &req.role_name,
            req.assigned_by.as_deref().or(Some(&user.0.id)),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let roles = user_repo
        .get_user_roles(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(roles))
}

// DELETE /api/v1/admin/users/{id}/roles/{role}
pub async fn remove_user_role(
    State(state): State<AppState>,
    Path((id, role)): Path<(String, String)>,
    _user: AuthenticatedUser,
) -> Result<Json<Vec<String>>, StatusCode> {
    let user_repo = UserRepository::new(state.db.clone());

    user_repo
        .remove_role(&id, &role)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let roles = user_repo
        .get_user_roles(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(roles))
}

// GET /api/v1/admin/users/{id}/stats
pub async fn get_user_stats(
    State(state): State<AppState>,
    Path(id): Path<String>,
    _user: AuthenticatedUser,
) -> Result<Json<UserStatsResponse>, StatusCode> {
    let user_repo = UserRepository::new(state.db.clone());

    let user = user_repo
        .get(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(UserStatsResponse {
        login_count: user.login_count,
        last_login_at: user.last_login_at,
        access_failed_count: user.access_failed_count,
        locked_out: user.locked_out,
        lockout_end: user.lockout_end,
        created_at: user.created_at,
    }))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/{id}", get(get_user).put(update_user).delete(delete_user))
        .route("/{id}/enable", post(enable_user))
        .route("/{id}/disable", post(disable_user))
        .route("/{id}/lock", post(lock_user))
        .route("/{id}/unlock", post(unlock_user))
        .route("/{id}/roles", get(get_user_roles).post(add_user_role))
        .route("/{id}/roles/{role}", delete(remove_user_role))
        .route("/{id}/stats", get(get_user_stats))
}

//! Auth service (trait + impl) — credential verification. `@injectable`-equivalent: shared
//! as `State<Arc<dyn IAuthService>>` (Rocket managed state is our DI container).

use async_trait::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::errors::{AppError, AppResult};
use crate::modules::access::models::user;

/// Authentication business logic (interface — controllers depend on this, not the impl).
#[async_trait]
pub trait IAuthService: Send + Sync {
    /// Verify credentials; returns the user on success, else `AppError` (401/403).
    async fn authenticate(
        &self,
        db: &DatabaseConnection,
        email: &str,
        password: &str,
    ) -> AppResult<user::Model>;
}

pub struct AuthService;

#[async_trait]
impl IAuthService for AuthService {
    async fn authenticate(
        &self,
        db: &DatabaseConnection,
        email: &str,
        password: &str,
    ) -> AppResult<user::Model> {
        let u = user::Entity::find()
            .filter(user::Column::Email.eq(email))
            .one(db)
            .await?
            .ok_or_else(|| AppError::unauthorized("Invalid credentials"))?;

        if u.blocked {
            return Err(AppError::forbidden("Account is blocked"));
        }
        if !bcrypt::verify(password, &u.password).unwrap_or(false) {
            return Err(AppError::unauthorized("Invalid credentials"));
        }
        Ok(u)
    }
}

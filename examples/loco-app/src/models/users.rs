//! The starter's user model, cut down to what sign-in needs.

use loco_rs::{auth::jwt, hash, prelude::*};
use serde_json::Map;
use uuid::Uuid;

pub use super::_entities::users::{self, ActiveModel, Entity, Model};

#[derive(Debug, Validate)]
pub struct Validator {
    #[validate(length(min = 2, message = "At least 2 characters."))]
    pub name: String,
    #[validate(email(message = "Enter an email address."))]
    pub email: String,
}

impl Validatable for ActiveModel {
    fn validator(&self) -> Box<dyn Validate> {
        Box::new(Validator {
            name: self.name.as_ref().to_owned(),
            email: self.email.as_ref().to_owned(),
        })
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        self.validate()?;
        if insert {
            let mut this = self;
            this.pid = ActiveValue::Set(Uuid::new_v4());
            this.api_key = ActiveValue::Set(format!("lo-{}", Uuid::new_v4()));
            Ok(this)
        } else {
            Ok(self)
        }
    }
}

#[async_trait::async_trait]
impl Authenticable for Model {
    async fn find_by_api_key(db: &DatabaseConnection, api_key: &str) -> ModelResult<Self> {
        let user = Entity::find()
            .filter(users::Column::ApiKey.eq(api_key))
            .one(db)
            .await?;
        user.ok_or(ModelError::EntityNotFound)
    }

    async fn find_by_claims_key(db: &DatabaseConnection, claims_key: &str) -> ModelResult<Self> {
        let pid = Uuid::parse_str(claims_key).map_err(|e| ModelError::Any(e.into()))?;
        let user = Entity::find()
            .filter(users::Column::Pid.eq(pid))
            .one(db)
            .await?;
        user.ok_or(ModelError::EntityNotFound)
    }
}

impl Model {
    /// The user with this email, if any.
    ///
    /// # Errors
    ///
    /// On a database error.
    pub async fn find_by_email(db: &DatabaseConnection, email: &str) -> ModelResult<Option<Self>> {
        Ok(Entity::find()
            .filter(users::Column::Email.eq(email))
            .one(db)
            .await?)
    }

    #[must_use]
    pub fn verify_password(&self, password: &str) -> bool {
        hash::verify_password(password, &self.password)
    }

    /// Insert a user with a hashed password.
    ///
    /// # Errors
    ///
    /// When the email is taken, validation fails or the database does.
    pub async fn create_with_password(
        db: &DatabaseConnection,
        name: &str,
        email: &str,
        password: &str,
    ) -> ModelResult<Self> {
        if Self::find_by_email(db, email).await?.is_some() {
            return Err(ModelError::EntityAlreadyExists);
        }
        let password = hash::hash_password(password).map_err(|e| ModelError::Any(e.into()))?;
        Ok(ActiveModel {
            email: ActiveValue::set(email.to_string()),
            password: ActiveValue::set(password),
            name: ActiveValue::set(name.to_string()),
            ..Default::default()
        }
        .insert(db)
        .await?)
    }

    /// A signed token for the `auth` cookie.
    ///
    /// # Errors
    ///
    /// When the token cannot be encoded.
    pub fn generate_jwt(&self, secret: &str, expiration: u64) -> ModelResult<String> {
        jwt::JWT::new(secret)
            .generate_token(expiration, self.pid.to_string(), Map::new())
            .map_err(ModelError::from)
    }
}

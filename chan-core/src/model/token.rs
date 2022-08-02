use crate::model::{User, UserStatus};
use crate::DBPool;
use async_graphql::{Error, Result};
use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, DecodingKey, Validation};
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
use tracing::{event, Level};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaim {
    pub issuer_user_id: String,
    pub expiration_time: DateTime<Utc>,
    // not_before_time: DateTime<Utc>,
    pub issued_at_time: DateTime<Utc>,
    pub token_uuid: Uuid,
}

impl TokenClaim {
    /// Verify token
    /// Returns Ok((true,userid)) if valid.
    /// Returns Err((false,None)) if invalid.
    pub fn verify(&self, db_pool: &DBPool) -> Result<(bool, Option<String>)> {
        //check time
        let datetime_now = Utc::now();
        debug_assert!(self.issued_at_time < datetime_now);
        if self.expiration_time < datetime_now {
            //expired
            return Ok((false, None));
        }

        //select a user with the uuid from DB
        let user = match User::select_by_user_id(&db_pool, &self.issuer_user_id)? {
            Some(u) => u,
            None => return Ok((false, None)), //user not found
        };

        //check user status

        let status: UserStatus = FromPrimitive::from_i32(user.user_status).unwrap();
        if status == UserStatus::Banned
            || status == UserStatus::Removed
            || status == UserStatus::Suspended
        {
            return Ok((false, None));
        }

        Ok((true, Some(user.id)))
    }
}

/// Verify token
/// Returns Ok(user_id) if valid.
/// Returns Err if token is invalid or not allowed.
/// GraphQL handler which uses this function should propagate the result by 'verify_token(...)?;'
pub fn verify_token(db_pool: &DBPool, token: &str) -> Result<String> {
    let secret_key = std::env::var("JWT_SECRET_KEY")?;
    let mut validation = Validation::default();
    validation.required_spec_claims.remove("exp");
    validation.validate_exp = false; //We use our own validation

    //verify token
    let token = decode::<TokenClaim>(
        &token,
        &DecodingKey::from_secret(secret_key.as_bytes()),
        &validation,
    )?;
    let (is_valid, user_uuid) = token.claims.verify(&db_pool)?;
    if !is_valid {
        assert!(user_uuid.is_none());
        return Err(Error::new("Invalid token."));
    }
    assert!(user_uuid.is_some());

    event!(Level::DEBUG, "verify_token OK!");

    Ok(user_uuid.unwrap())
}

use crate::model::TokenClaim;
use crate::schema::users;
use crate::DBPool;
use argon2::Config;
use async_graphql::{Error, Result, SimpleObject};
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use diesel::{Insertable, Queryable};
use jsonwebtoken::{encode, EncodingKey, Header};
use num_traits::{FromPrimitive, ToPrimitive};
use uuid::Uuid;

#[derive(FromPrimitive, PartialEq, Debug, ToPrimitive)]
pub enum UserType {
    Admin = 1,
    Moderator = 2,
    Normal = 3,
}

impl UserType {
    pub fn from_i32(user_type: i32) -> anyhow::Result<Self> {
        let user_type = FromPrimitive::from_i32(user_type);
        match user_type {
            Some(t) => Ok(t),
            None => Err(anyhow::Error::msg("Invalid user type number")),
        }
    }
}

//TODO: write tests
#[derive(FromPrimitive, PartialEq, Debug, ToPrimitive)]
pub enum UserStatus {
    Normal = 1,
    Suspended = 2,
    Banned = 3,
    Removed = 4, //User deleted account
}

impl UserStatus {
    pub fn from_i32(user_status: i32) -> anyhow::Result<Self> {
        let user_status = FromPrimitive::from_i32(user_status);
        match user_status {
            Some(t) => Ok(t),
            None => Err(anyhow::Error::msg("Invalid user status number")),
        }
    }
}

#[derive(Queryable, SimpleObject, Clone)]
pub struct User {
    #[graphql(skip)]
    primary_key: i32,
    pub id: String,
    pub registered_at: DateTime<Utc>,
    #[graphql(skip)]
    pub argon2_password: String,
    pub user_type: i32,
    pub user_status: i32,
}

impl User {
    /*
    pub fn select_by_uuid(db_pool: &DBPool, user_uuid: &Uuid) -> Result<Self> {
        use crate::schema::users::dsl::*;
        //check if a thread with the uuid is exist
        let user_vec = users
            .filter(uuid.eq(user_uuid))
            .limit(1)
            .load::<User>(&db_pool.get()?)?;
        if user_vec.len() != 1 {
            debug_assert_eq!(user_vec.len(), 0);
            return Err(Error::new("Invalid user uuid."));
        }
        Ok(user_vec[0].clone())
    }
    */

    // Look up users with the id from database
    pub fn select_by_user_id(db_pool: &DBPool, user_id: &str) -> Result<Option<Self>> {
        use crate::schema::users::dsl::*;
        let v = users
            .filter(id.eq(user_id))
            .limit(1)
            .load::<User>(&db_pool.get()?)?;
        debug_assert!(v.len() <= 1);
        if v.len() == 0 {
            Ok(None)
        } else {
            Ok(Some(v[0].clone()))
        }
    }

    /// Create a new user and insert to DB.(Registration)
    /// Returns the created user.
    pub fn create_new(db_pool: &DBPool, user_type: UserType, password: &str) -> Result<Self> {
        let argon2_config = Config::default();

        //check password length
        if password.len() < 16 {
            return Err(Error::new("Password is too short.(at least 16)"));
        }

        //hash password
        let mut salt = vec![0; 128];
        openssl::rand::rand_bytes(&mut salt).unwrap();
        let hashed_password =
            argon2::hash_encoded(&password.as_bytes(), &salt, &argon2_config).unwrap();
        let matches = argon2::verify_encoded(&hashed_password, &password.as_bytes()).unwrap();
        if !matches {
            return Err(Error::new("This should not happen."));
        }
        let mut user_id;

        //todo better rand
        loop {
            user_id = random_string::generate(16, "0123456789abcdefghijklmnopqrstuvwxyz");
            if User::select_by_user_id(&db_pool, &user_id)?.is_none() {
                break;
            }
        }
        let new_user = NewUser {
            id: &user_id,
            registered_at: &Utc::now(),
            argon2_password: &hashed_password,
            user_type: &user_type.to_i32().unwrap(), //nomal user
            user_status: &UserStatus::Normal.to_i32().unwrap(),
        };

        let created_user = diesel::insert_into(users::table)
            .values(&new_user)
            .get_result::<User>(&db_pool.get()?)
            .expect("sing_up failed");

        Ok(created_user)
    }

    pub fn login(db_pool: &DBPool, user_id: &str, password: &str) -> Result<String> {
        use crate::schema::users::dsl::*;
        //select user by user id
        let v = users
            .filter(id.eq(user_id))
            .limit(1)
            .load::<User>(&db_pool.get()?)?;
        if v.len() == 0 {
            return Err(Error::new("Invalid user id."));
        }
        let user = &v[0];
        //check password
        let matches = argon2::verify_encoded(&user.argon2_password, &password.as_bytes()).unwrap();
        if !matches {
            return Err(Error::new("Invalid password."));
        }
        //check account status
        let status: UserStatus = FromPrimitive::from_i32(user.user_status).unwrap();
        //TODO: what about suspended user, for now block
        if status == UserStatus::Banned
            || status == UserStatus::Removed
            || status == UserStatus::Suspended
        {
            return Err(Error::new("Not allowed."));
        }

        //generate token
        let secret_key = std::env::var("JWT_SECRET_KEY")?;
        let datetime_now = Utc::now();
        let token = encode(
            &Header::default(),
            &TokenClaim {
                issuer_user_id: user.id.to_owned(),
                expiration_time: datetime_now + Duration::hours(48), //expires after 48 hours(for now)
                token_uuid: Uuid::new_v4(),
                issued_at_time: datetime_now,
            },
            &EncodingKey::from_secret(secret_key.as_bytes()),
        )?;
        Ok(token)
    }

    pub fn change_status(db_pool: &DBPool, user_id: &str, status: UserStatus) -> Result<()> {
        assert!(status.to_i32().is_some());
        use crate::schema::users::dsl::*;

        diesel::update(users.filter(id.eq(user_id)))
            .set(user_status.eq(status.to_i32().unwrap()))
            .execute(&db_pool.get()?)?;
        Ok(())
    }

    pub fn change_type(db_pool: &DBPool, user_id: &str, new_type: UserType) -> Result<()> {
        assert!(new_type.to_i32().is_some());
        use crate::schema::users::dsl::*;
        diesel::update(users.filter(id.eq(user_id)))
            .set(user_type.eq(new_type.to_i32().unwrap()))
            .execute(&db_pool.get()?)?;
        Ok(())
    }
}

/// diesel model
#[derive(Insertable)]
#[table_name = "users"]
struct NewUser<'a> {
    pub id: &'a str,
    pub registered_at: &'a DateTime<Utc>,
    pub argon2_password: &'a str,
    pub user_type: &'a i32,
    pub user_status: &'a i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::FromPrimitive;
    use num_traits::ToPrimitive;
    #[test]
    fn user_type() {
        let admin = UserType::Admin;
        assert_eq!(admin.to_i32().unwrap(), 1);
        let tmp: UserType = FromPrimitive::from_i32(1).unwrap();
        assert_eq!(UserType::Admin, tmp);

        let mode = UserType::Moderator;
        assert_eq!(mode.to_i32().unwrap(), 2);
        let tmp: UserType = FromPrimitive::from_i32(2).unwrap();
        assert_eq!(UserType::Moderator, tmp);

        let normal = UserType::Normal;
        assert_eq!(normal.to_i32().unwrap(), 3);
        let tmp: UserType = FromPrimitive::from_i32(3).unwrap();
        assert_eq!(UserType::Normal, tmp);
    }

    #[test]
    #[should_panic]
    fn invalid_user_type() {
        let _: UserType = FromPrimitive::from_i32(4).unwrap();
    }
}

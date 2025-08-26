use std::error::Error;

use argon2::{
    password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};

use sqlx::PgPool;

use crate::domain::{
    data_stores::{UserStore, UserStoreError},
    Email, Password, User,
};

pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    // TODO: Implement all required methods. Note that you will need to make SQL queries against our PostgreSQL instance inside these methods.
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        match sqlx::query!(
            "INSERT INTO users (email, password_hash, requires_2fa) VALUES ($1, $2, $3) RETURNING email",
            user.email.as_ref(),
            compute_password_hash(user.password.as_ref()).await.unwrap(),
            user.requires_2fa
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(_) => (),
            Err(_) => return Err(UserStoreError::UserAlreadyExists),
        }

        Ok(())
        //if self.users.contains_key(&user.email) {
        //    Err(UserStoreError::UserAlreadyExists)
        //} else {
        //    self.users.insert(user.email.clone(), user);
        //    Ok(())
        //}
    }

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        let rec = match sqlx::query!(
            "select email, password_hash, requires_2fa from users where email = $1",
            email.as_ref(),
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(x) => x,
            Err(_) => return Err(UserStoreError::UserNotFound),
        };

        Ok(User {
            email: Email::parse(rec.email).unwrap(),
            password: Password::parse(rec.password_hash).unwrap(),
            requires_2fa: rec.requires_2fa,
        })
        //        match self.users.get(email) {
        //            Some(u) => Ok(u.clone()),
        //            None => Err(UserStoreError::UserNotFound),
        //        }
    }

    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        let rec = match sqlx::query!(
            "select email, password_hash, requires_2fa from users where email = $1",
            email.as_ref(),
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(x) => x,
            Err(_) => return Err(UserStoreError::UserNotFound),
        };

        if verify_password_hash(&rec.password_hash, password.as_ref())
            .await
            .is_ok()
        {
            Ok(())
        } else {
            Err(UserStoreError::InvalidCredentials)
        }
        //        match self.users.get(email) {
        //            Some(user) => {
        //                if user.password.as_ref() == password.as_ref() {
        //                    Ok(())
        //                } else {
        //                    Err(UserStoreError::InvalidCredentials)
        //                }
        //            }
        //            None => Err(UserStoreError::UserNotFound),
        //        }
    }
}

// Helper function to verify if a given password matches an expected hash
// TODO: Hashing is a CPU-intensive operation. To avoid blocking
// other async tasks, update this function to perform hashing on a
// separate thread pool using tokio::task::spawn_blocking. Note that you
// will need to update the input parameters to be String types instead of &str
async fn verify_password_hash(
    expected_password_hash: &str,
    password_candidate: &str,
) -> Result<(), Box<dyn Error>> {
    let expected_password_hash: PasswordHash<'_> = PasswordHash::new(expected_password_hash)?;

    Argon2::default()
        .verify_password(password_candidate.as_bytes(), &expected_password_hash)
        .map_err(|e| e.into())
}

// Helper function to hash passwords before persisting them in the database.
// TODO: Hashing is a CPU-intensive operation. To avoid blocking
// other async tasks, update this function to perform hashing on a
// separate thread pool using tokio::task::spawn_blocking. Note that you
// will need to update the input parameters to be String types instead of &str
async fn compute_password_hash(password: &str) -> Result<String, Box<dyn Error>> {
    let salt: SaltString = SaltString::generate(&mut rand::thread_rng());
    let password_hash = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, None)?,
    )
    .hash_password(password.as_bytes(), &salt)?
    .to_string();

    Ok(password_hash)
}

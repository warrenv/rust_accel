use color_eyre::eyre::{Context, Result};
use secrecy::{ExposeSecret, Secret};

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
    #[tracing::instrument(name = "Adding user to PostgreSQL", skip_all)]
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let password_hash = compute_password_hash(user.password.as_ref().to_owned())
            .await
            //.map_err(UserStoreError::UnexpectedError)?;
            .map_err(UserStoreError::UnexpectedError)?;

        //match sqlx::query!(
        sqlx::query!(
            "INSERT INTO users (email, password_hash, requires_2fa) VALUES ($1, $2, $3) RETURNING email",
            user.email.as_ref().expose_secret(),
            &password_hash.expose_secret(),
            //compute_password_hash(user.password.as_ref()).await.unwrap(),
            user.requires_2fa
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| UserStoreError::UnexpectedError(e.into()))?;
        //{
        //    Ok(_) => (),
        //    Err(_) => return Err(UserStoreError::UserAlreadyExists),
        //}

        Ok(())
        //if self.users.contains_key(&user.email) {
        //    Err(UserStoreError::UserAlreadyExists)
        //} else {
        //    self.users.insert(user.email.clone(), user);
        //    Ok(())
        //}
    }

    #[tracing::instrument(name = "Retrieving user from PostgreSQL", skip_all)]
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        let rec = match sqlx::query!(
            "select email, password_hash, requires_2fa from users where email = $1",
            email.as_ref().expose_secret(),
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(x) => x,
            Err(_) => return Err(UserStoreError::UserNotFound),
        };

        Ok(User {
            email: Email::parse(Secret::new(rec.email)).unwrap(),
            //            password: Password::parse(rec.password_hash).unwrap(),
            password: Password::parse(Secret::new(rec.password_hash))
                .map_err(UserStoreError::UnexpectedError)?,
            requires_2fa: rec.requires_2fa,
        })
        //        match self.users.get(email) {
        //            Some(u) => Ok(u.clone()),
        //            None => Err(UserStoreError::UserNotFound),
        //        }
    }

    #[tracing::instrument(name = "Validating user credentials in PostgreSQL", skip_all)]
    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        let rec = match sqlx::query!(
            "select email, password_hash, requires_2fa from users where email = $1",
            email.as_ref().expose_secret(),
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(x) => x,
            Err(_) => return Err(UserStoreError::UserNotFound),
        };

        if verify_password_hash(
            Secret::new(rec.password_hash),
            password.as_ref().to_owned(),
            //Secret::new(password.as_ref()), //.expose_secret(),
        )
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

#[tracing::instrument(name = "Verify password hash", skip_all)]
async fn verify_password_hash(
    expected_password_hash: Secret<String>,
    password_candidate: Secret<String>,
) -> Result<()> {
    let current_span: tracing::Span = tracing::Span::current();
    let result = tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let expected_password_hash: PasswordHash<'_> =
                PasswordHash::new(expected_password_hash.expose_secret())?;

            Argon2::default()
                .verify_password(
                    password_candidate.expose_secret().as_bytes(),
                    &expected_password_hash,
                )
                .wrap_err("failed to verify password hash")
        })
    })
    .await;

    result?
}

// Helper function to hash passwords before persisting them in the database.
// TODO: Hashing is a CPU-intensive operation. To avoid blocking
// other async tasks, update this function to perform hashing on a
// separate thread pool using tokio::task::spawn_blocking. Note that you
// will need to update the input parameters to be String types instead of &str

#[tracing::instrument(name = "Computing password hash", skip_all)]
async fn compute_password_hash(password: Secret<String>) -> Result<Secret<String>> {
    let current_span: tracing::Span = tracing::Span::current();

    let result = tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let salt: SaltString = SaltString::generate(&mut rand::thread_rng());
            let password_hash = Argon2::new(
                Algorithm::Argon2id,
                Version::V0x13,
                Params::new(15000, 2, 1, None)?,
            )
            .hash_password(password.expose_secret().as_bytes(), &salt)? // Updated!
            .to_string();

            Ok(Secret::new(password_hash))
        })
    })
    .await;

    result?
}

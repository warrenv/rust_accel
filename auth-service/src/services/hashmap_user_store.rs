use std::collections::HashMap;

use crate::domain::{User, UserStore, UserStoreError};

#[derive(Default)]
pub struct HashmapUserStore {
    users: HashMap<String, User>,
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(&user.email) {
            Err(UserStoreError::UserAlreadyExists)
        } else {
            self.users.insert(user.email.clone(), user);
            Ok(())
        }
    }

    async fn get_user(&self, email: &str) -> Result<User, UserStoreError> {
        match self.users.get(email) {
            Some(u) => Ok(u.clone()),
            None => Err(UserStoreError::UserNotFound),
        }
    }

    async fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        match self.users.get(email) {
            Some(user) => {
                if user.password == password {
                    Ok(())
                } else {
                    Err(UserStoreError::InvalidCredentials)
                }
            }
            None => Err(UserStoreError::UserNotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_user_succeeds_when_user_not_already_added() {
        let expected = Ok(());
        let mut store = HashmapUserStore::default();
        let user = User::new(
            "user@example.com".to_owned(),
            "password123".to_owned(),
            true,
        );

        let actual = store.add_user(user.clone()).await;

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_add_user_fails_when_user_already_added() {
        let expected = Err(UserStoreError::UserAlreadyExists);
        let mut store = HashmapUserStore::default();
        let user = User::new(
            "user@example.com".to_owned(),
            "password123".to_owned(),
            true,
        );

        let _ = store.add_user(user.clone()).await;
        let actual = store.add_user(user.clone()).await;

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_get_user_succeeds_when_user_exists() {
        let expected = User::new(
            "user@example.com".to_owned(),
            "password123".to_owned(),
            true,
        );
        let mut store = HashmapUserStore::default();

        let _ = store.add_user(expected.clone()).await;
        let actual = store.get_user(&expected.email).await.unwrap();

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_get_user_fails_when_user_does_not_exist() {
        let expected = Err(UserStoreError::UserNotFound);
        let store = HashmapUserStore::default();
        let user = User::new(
            "user@example.com".to_owned(),
            "password123".to_owned(),
            true,
        );

        let actual = store.get_user(&user.email).await;

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_validate_user_when_user_is_valid() {
        let expected = Ok(());
        let user = User::new(
            "user@example.com".to_owned(),
            "password123".to_owned(),
            true,
        );
        let mut store = HashmapUserStore::default();

        let _ = store.add_user(user.clone()).await;
        let actual = store.validate_user(&user.email, &user.password).await;

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_validate_user_when_user_does_not_exist() {
        let expected = Err(UserStoreError::UserNotFound);
        let user = User::new(
            "user@example.com".to_owned(),
            "password123".to_owned(),
            true,
        );
        let store = HashmapUserStore::default();

        let actual = store.validate_user(&user.email, &user.password).await;

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_validate_user_when_password_does_not_match() {
        let expected = Err(UserStoreError::InvalidCredentials);
        let user = User::new(
            "user@example.com".to_owned(),
            "password123".to_owned(),
            true,
        );
        let mut store = HashmapUserStore::default();

        let _ = store.add_user(user.clone()).await;
        let actual = store
            .validate_user(&user.email, &"non_matchin_password")
            .await;

        assert_eq!(actual, expected);
    }
}

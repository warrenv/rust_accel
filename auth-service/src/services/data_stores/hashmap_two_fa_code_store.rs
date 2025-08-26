use std::collections::HashMap;

use crate::domain::{
    data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError},
    email::Email,
};

#[derive(Default)]
pub struct HashmapTwoFACodeStore {
    codes: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

#[async_trait::async_trait]
impl TwoFACodeStore for HashmapTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        self.codes.insert(email, (login_attempt_id, code));
        Ok(())
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        match self.codes.remove(email) {
            Some(_) => Ok(()),
            None => Err(TwoFACodeStoreError::UnexpectedError),
        }
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        match self.codes.get(email) {
            Some(x) => Ok(x.clone()),
            None => Err(TwoFACodeStoreError::UnexpectedError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_code_succeeds() {
        let expected = Ok(());
        let mut store = HashmapTwoFACodeStore::default();
        let email = Email::parse("user@example.com".to_owned()).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let two_fa_code = TwoFACode::default();

        let actual = store.add_code(email, login_attempt_id, two_fa_code).await;

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_remove_code_successful_when_code_exists() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = Email::parse("user@example.com".to_owned()).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let two_fa_code = TwoFACode::default();

        {
            let expected = Ok(());
            let actual = store
                .add_code(email.clone(), login_attempt_id, two_fa_code)
                .await;
            assert_eq!(actual, expected);
        }

        {
            let expected = Ok(());
            let actual = store.remove_code(&email).await;

            assert_eq!(actual, expected);
        }
    }

    #[tokio::test]
    async fn test_remove_code_errors_when_code_does_not_exist() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = Email::parse("user@example.com".to_owned()).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let two_fa_code = TwoFACode::default();

        {
            let expected = true;
            let actual = store.remove_code(&email).await;

            assert_eq!(actual.is_err(), expected);
        }
    }

    #[tokio::test]
    async fn test_get_code_succeeds() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = Email::parse("user@example.com".to_owned()).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let two_fa_code = TwoFACode::default();

        {
            let expected = Ok(());
            let actual = store
                .add_code(email.clone(), login_attempt_id.clone(), two_fa_code.clone())
                .await;

            assert_eq!(actual, expected);
        }

        {
            let expected = true;
            let actual = store.get_code(&email.clone()).await;

            assert_eq!(actual.is_ok(), expected);
            assert_eq!(actual.clone().unwrap().0, login_attempt_id);
            assert_eq!(actual.clone().unwrap().1, two_fa_code);
        }
    }

    #[tokio::test]
    async fn test_get_code_fails_when_email_does_not_exist() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = Email::parse("user@example.com".to_owned()).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let two_fa_code = TwoFACode::default();

        let expected = true;
        let actual = store.get_code(&email.clone()).await;

        assert_eq!(actual.is_err(), expected);
    }
}

use crate::domain::{Email, Password};
use secrecy::Secret;

#[derive(PartialEq, Clone, Debug)]
pub struct User {
    pub email: Email,
    pub password: Password,
    pub requires_2fa: bool,
}

impl User {
    pub fn new(email: Email, password: Password, requires_2fa: bool) -> Self {
        Self {
            email,
            password,
            requires_2fa,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_returns_a_user() {
        let expected = User::new(
            Email::parse(Secret::new("foo@example.com".to_string())).unwrap(),
            Password::parse(Secret::new("password123".to_string())).unwrap(),
            true,
        );

        let actual = User::new(
            Email::parse(Secret::new("foo@example.com".to_string())).unwrap(),
            Password::parse(Secret::new("password123".to_string())).unwrap(),
            true,
        );

        assert_eq!(actual, expected);
    }
}

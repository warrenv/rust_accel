//use crate::domain::{email::Email, password::Password};
use crate::domain::{Email, Password};

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
            Email::parse("foo@example.com".to_owned()).unwrap(),
            Password::parse("password123".to_owned()).unwrap(),
            true,
        );

        let actual = User::new(
            Email::parse("foo@example.com".to_owned()).unwrap(),
            Password::parse("password123".to_owned()).unwrap(),
            true,
        );

        assert_eq!(actual, expected);
    }
}

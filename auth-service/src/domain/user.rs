#[derive(PartialEq, Clone, Debug)]
pub struct User {
    pub email: String,
    pub password: String,
    pub requires_2fa: bool,
}

impl User {
    pub fn new(email: String, password: String, requires_2fa: bool) -> Self {
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
        assert_eq!(
            User::new(
                "foo@example.com".to_owned(),
                "password123".to_owned(),
                false
            )
            .email,
            "foo@example.com"
        );
    }
}

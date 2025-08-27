use color_eyre::eyre::{eyre, Result};
use secrecy::{ExposeSecret, Secret};

use std::hash::Hash;

#[derive(Clone, Debug)]
pub struct Email(Secret<String>);

impl PartialEq for Email {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Hash for Email {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.expose_secret().hash(state);
    }
}

impl Eq for Email {}

impl Email {
    pub fn parse(email: Secret<String>) -> Result<Self> {
        match email.expose_secret() != "" && email.expose_secret().contains("@") {
            true => Ok(Self(email)),
            false => Err(eyre!("Failed to parse string to a Password type")), // Err("invalid password".to_owned().into()),
        }
    }
}

impl AsRef<Secret<String>> for Email {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_returns_ok_given_valid_email() {
        assert_eq!(
            Email::parse(Secret::new("foo@example.com".to_string())).is_ok(),
            true
        );
    }

    #[test]
    fn test_parse_returns_err_given_invalid_email() {
        assert_eq!(
            Email::parse(Secret::new("fooexample.com".to_string())).is_err(),
            true
        );
    }

    #[test]
    fn test_parse_returns_err_given_empty_email() {
        assert_eq!(Email::parse(Secret::new("".to_string())).is_err(), true);
    }

    #[test]
    fn test_as_ref_works() {
        let expected = "foo@example.com";

        //let email = Email::parse(Secret::new("foo@example.com").to_owned()).unwrap();
        let email = Email::parse(Secret::new("foo@example.com".to_string())).unwrap();
        let actual = email.as_ref().expose_secret();

        assert_eq!(expected, actual);
    }
}

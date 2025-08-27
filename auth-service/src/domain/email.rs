use color_eyre::eyre::{eyre, Result};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Email(String);

impl Email {
    pub fn parse(email: String) -> Result<Self> {
        match email != "" && email.contains("@") {
            true => Ok(Self(email)),
            false => Err(eyre!("Failed to parse string to a Password type")), // Err("invalid password".to_owned().into()),
        }
    }
}

impl AsRef<String> for Email {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_returns_ok_given_valid_email() {
        assert_eq!(Email::parse("foo@example.com".to_owned(),).is_ok(), true);
    }

    #[test]
    fn test_parse_returns_err_given_invalid_email() {
        assert_eq!(Email::parse("fooexample.com".to_owned(),).is_err(), true);
    }

    #[test]
    fn test_parse_returns_err_given_empty_email() {
        assert_eq!(Email::parse("".to_owned(),).is_err(), true);
    }

    #[test]
    fn test_as_ref_works() {
        let expected = "foo@example.com";

        let email = Email::parse("foo@example.com".to_owned()).unwrap();
        let actual = email.as_ref();

        assert_eq!(expected, actual);
    }
}

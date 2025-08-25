use std::error::Error;

#[derive(Clone, PartialEq, Debug)]
pub struct Password(String);

impl Password {
    pub fn parse(password: String) -> Result<Self, Box<dyn Error>> {
        match password.len() >= 8 {
            true => Ok(Self(password)),
            false => Err("invalid password".to_owned().into()),
        }
    }
}

impl AsRef<String> for Password {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_returns_ok_given_valid_password() {
        let expected = true;
        let actual = Password::parse("12345678".to_owned()).is_ok();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_returns_err_given_invalid_password() {
        let expected = true;
        let actual = Password::parse("123".to_owned()).is_err();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_as_ref_works() {
        let expected = "password12345";

        let password = Password::parse("password12345".to_owned()).unwrap();
        let actual = password.as_ref();

        assert_eq!(expected, actual);
    }
}

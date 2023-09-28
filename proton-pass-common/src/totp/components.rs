use crate::totp::algorithm::Algorithm;
use crate::totp::error::TOTPError;
use crate::totp::get_value::{GetQueryValue, Queries};
use queryst::parse;
use uriparse::URI;

#[derive(Debug)]
pub struct TOTPComponents {
    pub label: Option<String>,
    pub secret: String,
    pub issuer: Option<String>,
    pub algorithm: Option<Algorithm>,
    pub digits: Option<u8>,
    pub period: Option<u16>,
}

impl TOTPComponents {
    pub fn from_uri(uri: &str) -> Result<Self, TOTPError> {
        match URI::try_from(uri) {
            Ok(uri) => Self::parse_uri(uri),
            Err(error) => Err(TOTPError::URIError(error)),
        }
    }

    fn parse_uri(uri: URI) -> Result<Self, TOTPError> {
        Self::check_scheme(&uri)?;
        Self::check_otp_type(&uri)?;

        let label = Self::parse_label(&uri);

        let queries = &Self::parse_queries(&uri)?;
        let secret = Self::get_secret(queries)?;
        let issuer = queries.get_string_value("issuer");
        let algorithm: Option<Algorithm> = Self::get_algorithm(queries)?;
        let digits: Option<u8> = queries.get_string_parsable_value("digits");
        let period: Option<u16> = queries.get_string_parsable_value("period");

        Ok(Self {
            label,
            secret,
            issuer,
            algorithm,
            digits,
            period,
        })
    }
}

impl TOTPComponents {
    fn check_scheme(uri: &URI) -> Result<(), TOTPError> {
        let scheme = uri.scheme().to_string();
        if scheme.to_lowercase() == "otpauth" {
            Ok(())
        } else {
            Err(TOTPError::InvalidScheme(scheme))
        }
    }

    fn check_otp_type(uri: &URI) -> Result<(), TOTPError> {
        match uri.authority() {
            Some(value) => {
                let authority = value.to_string();
                if authority.is_empty() {
                    Err(TOTPError::NoAuthority)
                } else if authority.to_lowercase() == "totp" {
                    Ok(())
                } else {
                    Err(TOTPError::InvalidAuthority(authority))
                }
            }
            None => Err(TOTPError::NoAuthority),
        }
    }

    fn parse_label(uri: &URI) -> Option<String> {
        match uri.path().segments().last() {
            Some(value) => {
                let label = value.to_string();
                if label.is_empty() {
                    None
                } else {
                    Some(label)
                }
            }
            _ => None,
        }
    }

    fn parse_queries(uri: &URI) -> Result<Queries, TOTPError> {
        let queries_string;
        if let Some(value) = uri.query() {
            queries_string = value.as_str();
        } else {
            return Err(TOTPError::NoQueries);
        }

        let queries_value;
        if let Ok(value) = parse(queries_string) {
            queries_value = value;
        } else {
            return Err(TOTPError::NoQueries);
        }

        match queries_value.as_object() {
            Some(value) => Ok(value.clone()),
            _ => Err(TOTPError::NoQueries),
        }
    }

    fn get_secret(queries: &Queries) -> Result<String, TOTPError> {
        match queries.get_string_value("secret") {
            Some(value) => {
                if value.is_empty() {
                    Err(TOTPError::EmptySecret)
                } else {
                    Ok(value)
                }
            }
            _ => Err(TOTPError::NoSecret),
        }
    }

    fn get_algorithm(queries: &Queries) -> Result<Option<Algorithm>, TOTPError> {
        match queries.get_string_value("algorithm") {
            Some(value) => match Algorithm::new(value.as_str()) {
                Ok(algorithm) => Ok(Some(algorithm)),
                Err(error) => Err(error),
            },
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::totp::algorithm::Algorithm::SHA512;
    use crate::totp::components::TOTPComponents;
    use crate::totp::error::TOTPError;

    fn make_sut(uri: &str) -> Result<TOTPComponents, TOTPError> {
        TOTPComponents::from_uri(uri)
    }

    #[test]
    fn invalid_scheme() {
        // Given
        let uri = "https://totp/john.doe%40example.com?secret=somesecret&algorithm=SHA1&digits=8&period=30";

        // When
        let sut = make_sut(uri);

        // Then
        match sut {
            Err(error) => assert_eq!(error, TOTPError::InvalidScheme("https".to_string())),
            _ => panic!("Should not be able to parse"),
        }
    }

    #[test]
    fn invalid_authority() {
        // Given
        let uri = "otpauth://hotp/john.doe%40example.com?secret=somesecret&algorithm=SHA1&digits=8&period=30";

        // When
        let sut = make_sut(uri);

        // Then
        match sut {
            Err(error) => assert_eq!(error, TOTPError::InvalidAuthority("hotp".to_string())),
            _ => panic!("Should not be able to parse"),
        }
    }

    #[test]
    fn no_authority() {
        // Given
        let uri = "otpauth://?secret=somesecret&algorithm=SHA1&digits=8&period=30";

        // When
        let sut = make_sut(uri);

        // Then
        match sut {
            Err(error) => assert_eq!(error, TOTPError::NoAuthority),
            _ => panic!("Should not be able to parse"),
        }
    }

    #[test]
    fn no_queries() {
        // Given
        let uri = "otpauth://totp/";

        // When
        let sut = make_sut(uri);

        // Then
        match sut {
            Err(error) => assert_eq!(error, TOTPError::NoQueries),
            _ => panic!("Should not be able to parse"),
        }
    }

    #[test]
    fn no_secret() {
        // Given
        let uri = "otpauth://totp/john.doe%40example.com?algorithm=SHA1&digits=8&period=30";

        // When
        let sut = make_sut(uri);

        // Then
        match sut {
            Err(error) => assert_eq!(error, TOTPError::NoSecret),
            _ => panic!("Should not be able to parse"),
        }
    }

    #[test]
    fn empty_secret() {
        // Given
        let uri = "otpauth://totp/john.doe%40example.com?secret=&algorithm=SHA1&digits=8&period=30";

        // When
        let sut = make_sut(uri);

        // Then
        match sut {
            Err(error) => assert_eq!(error, TOTPError::EmptySecret),
            _ => panic!("Should not be able to parse"),
        }
    }

    #[test]
    fn invalid_algorithm() {
        // Given
        let uri = "otpauth://totp/john.doe%40example.com?secret=somesecret&algorithm=SHA128&digits=8&period=30";

        // When
        let sut = make_sut(uri);

        // Then
        match sut {
            Err(error) => assert_eq!(error, TOTPError::InvalidAlgorithm("SHA128".to_string())),
            _ => panic!("Should not be able to parse"),
        }
    }

    #[test]
    fn explicit_params() {
        // Given
        let uri =
            "otpauth://totp/john.doe%40example.com?secret=somesecret&issuer=ProtonMail&algorithm=SHA512&digits=8&period=45";

        // When
        let sut = make_sut(uri);

        // Then
        match sut {
            Ok(components) => {
                assert_eq!(components.label, Some("john.doe%40example.com".to_string()));
                assert_eq!(components.secret, "somesecret");
                assert_eq!(components.issuer, Some("ProtonMail".to_string()));
                assert_eq!(components.algorithm, Some(SHA512));
                assert_eq!(components.digits, Some(8));
                assert_eq!(components.period, Some(45));
            }
            _ => panic!("Should be able to parse"),
        }
    }

    #[test]
    fn implicit_params() {
        // Given
        let uri = "otpauth://totp/?secret=somesecret";

        // When
        let sut = make_sut(uri);

        // Then
        match sut {
            Ok(components) => {
                assert_eq!(components.label, None);
                assert_eq!(components.secret, "somesecret");
                assert_eq!(components.issuer, None);
                assert_eq!(components.algorithm, None);
                assert_eq!(components.digits, None);
                assert_eq!(components.period, None);
            }
            _ => panic!("Should be able to parse"),
        }
    }
}

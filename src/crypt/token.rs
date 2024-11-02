use std::fmt::Display;
use std::str::FromStr;

use crate::config;
use crate::crypt::{encrypt_into_b64u, Error, Result};
use crate::util::{b64u_decode, b64u_encode, now_utc, now_utc_plus_sec_str, parse_utc};

#[derive(Debug)]
pub struct Token {
    pub ident: String,
    pub exp: String,
    pub sign_b64u: String,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Token(ident: {}, exp: {}, sign_b64u: {})",
            b64u_encode(&self.ident),
            b64u_encode(&self.exp),
            self.sign_b64u
        )
    }
}

impl FromStr for Token {
    type Err = Error;

    fn from_str(token_str: &str) -> Result<Self> {
        let parts: Vec<&str> = token_str.split('.').collect();
        if parts.len() != 3 {
            return Err(Error::TokenInvalidFormat);
        }

        let ident = b64u_decode(parts[0]).map_err(|_| Error::TokenCannotDecodeIdent)?;
        let exp = b64u_decode(parts[1]).map_err(|_| Error::TokenCannotDecodeExp)?;
        let sign_b64u = parts[2].to_string();

        Ok(Token {
            ident,
            exp,
            sign_b64u,
        })
    }
}

pub fn generate_web_token(user: &str, salt: &str) -> Result<Token> {
    let config = &config();
    _generate_token(user, config.TOKEN_DURATION_SEC, salt, &config.TOKEN_KEY)
}

pub fn validate_web_token(original_token: &Token, salt: &str) -> Result<()> {
    let config = &config();
    _validate_token_sign_and_exp(original_token, salt, &config.TOKEN_KEY)
}

fn _generate_token(ident: &str, duration_sec: f64, salt: &str, key: &[u8]) -> Result<Token> {
    let ident = ident.to_string();
    let exp = now_utc_plus_sec_str(duration_sec);

    let sign_b64u = _token_sign_into_b64u(&ident, &exp, salt, key)?;

    Ok(Token {
        ident,
        exp,
        sign_b64u,
    })
}

fn _validate_token_sign_and_exp(original_token: &Token, salt: &str, key: &[u8]) -> Result<()> {
    let new_sign_b64u =
        _token_sign_into_b64u(&original_token.ident, &original_token.exp, salt, key)?;

    if new_sign_b64u != original_token.sign_b64u {
        return Err(Error::TokenSignatureNotMatch);
    }

    let original_exp = parse_utc(&original_token.exp).map_err(|_| Error::TokenExpNotIso)?;
    let now = now_utc();

    if original_exp < now {
        return Err(Error::TokenExpired);
    }

    Ok(())
}

fn _token_sign_into_b64u(ident: &str, exp: &str, salt: &str, key: &[u8]) -> Result<String> {
    let content = format!("{}.{}", ident, exp);
    let enc_content = crate::crypt::EncryptContent {
        content,
        salt: salt.to_string(),
    };

    let signature = encrypt_into_b64u(key, &enc_content)?;

    Ok(signature)
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use super::*;
    use anyhow::Result;

    #[test]
    fn test_display_token_ok() -> Result<()> {
        let fx_token = Token {
            ident: "ident".to_string(),
            exp: "exp".to_string(),
            sign_b64u: "sign_b64u".to_string(),
        };

        println!("->> {}", fx_token);

        Ok(())
    }

    #[test]
    fn test_validate_web_token_ok() -> Result<()> {
        let fx_user = "user";
        let fx_salt = "salt";
        let fx_duration_sec = 0.02;
        let token_key = &config().TOKEN_KEY;
        let fx_token = _generate_token(fx_user, fx_duration_sec, fx_salt, token_key)?;

        thread::sleep(Duration::from_millis(10));
        let res = validate_web_token(&fx_token, fx_salt);

        res?;

        Ok(())
    }

    #[test]
    fn test_validate_web_token_err_expired() -> Result<()> {
        let fx_user = "user";
        let fx_salt = "salt";
        let fx_duration_sec = 0.01;
        let token_key = &config().TOKEN_KEY;
        let fx_token = _generate_token(fx_user, fx_duration_sec, fx_salt, token_key)?;

        thread::sleep(Duration::from_millis(20));
        let res = validate_web_token(&fx_token, fx_salt);

        assert!(
            matches!(res, Err(Error::TokenExpired)),
            "Should have matched Err(Error::TokenExpired) but was `{res:?}`"
        );

        Ok(())
    }
}

use crate::config;
use crate::crypt::{
    encrypt_into_b64u,
    error::{Error, Result},
    EncryptContent,
};

pub fn encrypt_pwd(enc_content: &EncryptContent) -> Result<String> {
    let key = &config().PWD_KEY;

    let encrypted = encrypt_into_b64u(&key, enc_content)?;

    Ok(format!("#01#{encrypted}"))
}

pub fn validate_pwd(enc_content: &EncryptContent, encrypted: &str) -> Result<()> {
    let expected = encrypt_pwd(enc_content)?;

    let encrypted = encrypted.trim_start_matches("#01#");

    if encrypted != expected {
        return Err(Error::PasswordNotMatch);
    }

    Ok(())
}

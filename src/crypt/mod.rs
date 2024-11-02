mod error;
pub mod pwd;
pub mod token;

pub use self::error::{Error, Result};

use hmac::{Hmac, Mac};
use sha2::Sha512;

pub struct EncryptContent {
    pub content: String,
    pub salt: String,
}

pub fn encrypt_into_b64u(key: &[u8], enc_content: &EncryptContent) -> Result<String> {
    let EncryptContent { content, salt } = enc_content;

    let mut hmac_sha512 = Hmac::<Sha512>::new_from_slice(key).map_err(|_| Error::KeyFailHmac)?;

    hmac_sha512.update(content.as_bytes());
    hmac_sha512.update(salt.as_bytes());

    let hmac_result = hmac_sha512.finalize().into_bytes();

    let result = base64_url::encode(&hmac_result);

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use rand::RngCore;

    #[test]
    fn test_encrypt_into_b64u_ok() -> Result<()> {
        let mut key = [0u8; 64];
        rand::thread_rng().fill_bytes(&mut key);

        let enc_content = EncryptContent {
            content: "content".to_string(),
            salt: "salt".to_string(),
        };

        let fx_res = encrypt_into_b64u(&key, &enc_content)?;

        let res = encrypt_into_b64u(&key, &enc_content)?;

        assert_eq!(fx_res, res);

        Ok(())
    }
}
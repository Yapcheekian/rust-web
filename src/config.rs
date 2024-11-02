use crate::{Error, Result};
use std::env;
use std::str::FromStr;
use std::sync::OnceLock;

pub fn config() -> &'static Config {
    static INSTANCE: OnceLock<Config> = OnceLock::new();

    INSTANCE.get_or_init(|| Config::load_from_env().unwrap_or_else(|e| panic!("{:?}", e)))
}

#[allow(non_snake_case)]
pub struct Config {
    pub WEB_FOLDER: String,
    pub DB_URL: String,
    pub PWD_KEY: Vec<u8>,
    pub TOKEN_KEY: Vec<u8>,
    pub TOKEN_DURATION_SEC: f64,
}

impl Config {
    fn load_from_env() -> Result<Self> {
        Ok(Self {
            WEB_FOLDER: get_env("SERVICE_WEB_FOLDER")?,
            DB_URL: get_env("SERVICE_DB_URL")?,
            PWD_KEY: get_env_b64u_as_u8s("SERVICE_PWD_KEY")?,
            TOKEN_KEY: get_env_b64u_as_u8s("SERVICE_PWD_KEY")?,
            TOKEN_DURATION_SEC: gen_env_parse::<f64>("SERVICE_TOKEN_DURATION_SEC")?,
        })
    }
}

fn get_env(name: &'static str) -> Result<String> {
    env::var(name).map_err(|_| Error::ConfigMissingEnv(name))
}

fn get_env_b64u_as_u8s(name: &'static str) -> Result<Vec<u8>> {
    let b64u = get_env(name)?;
    base64_url::decode(&b64u).map_err(|_| Error::ConfigWrongFormat(name))
}

fn gen_env_parse<T: FromStr>(name: &'static str) -> Result<T> {
    let val = get_env(name)?;

    val.parse::<T>().map_err(|_| Error::ConfigWrongFormat(name))
}

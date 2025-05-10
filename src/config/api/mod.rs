use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    nlgo: Credentials,
    aladin: Credentials,
    naver: Credentials,
    kyobo: Credentials,
}

#[derive(Debug, Deserialize)]
pub struct Credentials {
    key: String,
    secret: Option<String>
}

impl Config {
    pub fn nlgo(&self) -> &Credentials {
        &self.nlgo
    }

    pub fn aladin(&self) -> &Credentials {
        &self.aladin
    }

    pub fn naver(&self) -> &Credentials {
        &self.naver
    }

    pub fn kyobo(&self) -> &Credentials {
        &self.kyobo
    }
}

impl Credentials {
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn secret(&self) -> Option<&str> {
        match &self.secret {
            None => None,
            Some(s) => Some(s.as_str())
        }
    }
}
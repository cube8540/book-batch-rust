use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct API {
    nlgo: Credentials,
    aladin: Credentials
}

#[derive(Debug, Deserialize)]
pub struct Credentials {
    key: String,
    secret: Option<String>
}

impl API {
    pub fn nlgo(&self) -> &Credentials {
        &self.nlgo
    }

    pub fn aladin(&self) -> &Credentials {
        &self.aladin
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
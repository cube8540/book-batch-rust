use crate::provider::html::kyobo::LoginProvider;
use crate::provider::html::ParsingError;
use std::env;
use std::env::VarError;
use thirtyfour_sync::prelude::ElementQueryable;
use thirtyfour_sync::{By, DesiredCapabilities, WebDriver, WebDriverCommands};

const AGENT: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/80.0.3987.149 Safari/537.36";

const COOKIE_DOMAIN: &'static str = ".kyobobook.co.kr";
const LOGIN_URL: &'static str = "https://mmbr.kyobobook.co.kr/login";

pub struct ChromeDriverLoginProvider {
    server_url: String,
    id: String,
    pw: String,

    access_token: Option<String>,
    last_login_at: Option<chrono::NaiveDateTime>,
}

pub fn new_provider() -> Result<ChromeDriverLoginProvider, VarError> {
    let id = env::var("KYOBO_ID")?;
    let pw = env::var("KYOBO_SECRET")?;

    let server_url = env::var("CHROMEDRIVER_URL")?;

    Ok(ChromeDriverLoginProvider {
        server_url,
        id,
        pw,
        access_token: None,
        last_login_at: None,
    })
}

impl LoginProvider for ChromeDriverLoginProvider {
    type CookieValue = String;

    fn login(&mut self) -> Result<(), ParsingError> {
        let mut caps = DesiredCapabilities::chrome();
        caps.add_chrome_arg(&format!("--user-agent={}", AGENT))
            .map_err(|err| ParsingError::AuthenticationError(err.to_string()))?;
        caps.add_chrome_arg("--disable-blink-features=AutomationControlled")
            .map_err(|err| ParsingError::AuthenticationError(err.to_string()))?;

        let driver = WebDriver::new(&self.server_url, caps)
            .map_err(|err| ParsingError::UnknownError(err.to_string()))?;
        driver.get(LOGIN_URL)
            .map_err(|err| ParsingError::PageNotFound(err.to_string()))?;

        let id_form = driver.find_element(By::ClassName("id"))
            .map_err(|err| ParsingError::ElementNotFound(err.to_string()))?;
        let id_element = id_form.find_element(By::ClassName("form_ip"))
            .map_err(|err| ParsingError::ElementNotFound(err.to_string()))?;

        let pw_form = driver.find_element(By::ClassName("pw"))
            .map_err(|err| ParsingError::ElementNotFound(err.to_string()))?;
        let pw_element = pw_form.find_element(By::ClassName("form_ip"))
            .map_err(|err| ParsingError::ElementNotFound(err.to_string()))?;

        _ = id_element.send_keys(&self.id)
            .map_err(|err| ParsingError::UnknownError(err.to_string()))?;
        _ = pw_element.send_keys(&self.pw)
            .map_err(|err| ParsingError::UnknownError(err.to_string()))?;

        let login_btn = driver.find_element(By::Id("loginBtn"))
            .map_err(|err| ParsingError::ElementNotFound(err.to_string()))?;
        _ = login_btn.click()
            .map_err(|err| ParsingError::UnknownError(err.to_string()))?;

        // 로그인 완료 대기
        let body = driver.query(By::ClassName("font-body"));
        body.first().unwrap().text().unwrap();

        let access_token = driver.get_cookie("accessToken")
            .map_err(|err| ParsingError::UnknownError(err.to_string()))?;

        let token = access_token.value().to_string().trim_matches('"').to_string();
        _ = driver.quit()
            .map_err(|err| ParsingError::UnknownError(err.to_string()))?;
        
        self.access_token = Some(token);
        self.last_login_at = Some(chrono::Local::now().naive_local());
        Ok(())
    }

    fn get_cookies(&self) -> Result<Vec<Self::CookieValue>, ParsingError> {
        if let Some(token) = self.access_token.as_ref() {
            let access_token = format!("accessToken={}; Domain={}; Path=/; Secure", token, COOKIE_DOMAIN);
            Ok(vec![access_token])
        } else {
            Err(ParsingError::UnknownError("Access token is None".to_owned()))
        }
    }
}
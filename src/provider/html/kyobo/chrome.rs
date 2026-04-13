use std::any::Any;
use crate::provider::html::kyobo::LoginProvider;
use crate::provider::html::ParsingError;
use headless_chrome::{Browser, LaunchOptions};
use std::env::VarError;
use std::{env, thread};
use std::ops::Add;
use headless_chrome::browser::tab::point::Point;

const AGENT: &'static str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Safari/537.36";

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

    let mut provider = ChromeDriverLoginProvider {
        server_url,
        id,
        pw,
        access_token: None,
        last_login_at: None,
    };
    provider.login().unwrap();
    Ok(provider)
}

impl LoginProvider for ChromeDriverLoginProvider {
    type CookieValue = String;

    fn login(&mut self) -> Result<(), ParsingError> {
        let user_agent = format!("--user-agent={}", AGENT);
        let options = LaunchOptions {
            headless: true,
            args: vec![
                user_agent.as_str(),
                "--disable-blink-features=AutomationControlled", // 자동화 플래그 비활성화
                "--disable-infobars",
                "--disable-dev-shm-usage",
                "--disable-renderer-backgrounding",
                "--disable-background-timer-throttling"
            ].into_iter().map(std::ffi::OsStr::new).collect(),
            ..Default::default()
        };

        let browser = Browser::new(options)
            .map_err(|e| ParsingError::UnknownError(e.to_string()))?;
        let tab = browser.new_tab()
            .map_err(|e| ParsingError::UnknownError(e.to_string()))?;

        tab.navigate_to(LOGIN_URL).map_err(|e| ParsingError::UnknownError(e.to_string()))?;
        tab.wait_until_navigated().map_err(|e| ParsingError::UnknownError(e.to_string()))?;

        tab.press_key("Tab").map_err(|e| ParsingError::UnknownError(e.to_string()))?;
        tab.press_key("Tab").map_err(|e| ParsingError::UnknownError(e.to_string()))?;
        tab.press_key("Tab").map_err(|e| ParsingError::UnknownError(e.to_string()))?;
        tab.type_str(self.id.as_str()).map_err(|e| ParsingError::UnknownError(e.to_string()))?;
        tab.press_key("Tab").map_err(|e| ParsingError::UnknownError(e.to_string()))?;
        tab.type_str(&self.pw).map_err(|e| ParsingError::UnknownError(e.to_string()))?;

        let login_btn = tab.wait_for_element("#loginBtn")
            .map_err(|_| ParsingError::ElementNotFound("login button cannot found".to_owned()))?;
        let point = login_btn.get_box_model().map_err(|e| ParsingError::UnknownError(e.to_string()))?;
        let new_point = point.content.top_left.add(Point{ x: 10.0, y: 10.0 });
        tab.move_mouse_to_point(new_point).map_err(|e| ParsingError::UnknownError(e.to_string()))?;
        tab.click_point(new_point).map_err(|e| ParsingError::UnknownError(e.to_string()))?;

        _ = tab.wait_for_elements(".font-body")
            .map_err(|_| ParsingError::ElementNotFound("login complete tag cannot found".to_owned()))?;

        let access_token = match tab.get_cookies() {
            Ok(cookies) => cookies.iter().find(|cookie| cookie.name == "accessToken").map(|cookie| cookie.value.to_string()),
            Err(err) => {
                return Err(ParsingError::UnknownError(err.to_string()));
            }
        };

        match access_token {
            Some(token) => {
                self.access_token = Some(token);
                self.last_login_at = Some(chrono::Local::now().naive_local());
                Ok(())
            }
            None => Err(ParsingError::AuthenticationError("token is not found".to_owned()))
        }
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
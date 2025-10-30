use anyhow::{Context, Result};
use headless_chrome::{Browser, LaunchOptions};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::config::KnxConfig;
use crate::device::{Device, DeviceType};

#[derive(Debug)]
pub struct KnxClient {
    client: reqwest::Client,
    config: Arc<KnxConfig>,
    session_id: Arc<RwLock<String>>,
}

impl KnxClient {
    pub fn new(config: Arc<KnxConfig>) -> Result<Self> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .context("Failed to create HTTP client")?;

        let session_id = Arc::new(RwLock::new(String::new()));

        Ok(Self { client, config, session_id })
    }

    pub async fn validate_session(&self) -> Result<bool> {
        let url = {
            let session_id = self.session_id.read().await;
            format!(
                "{}/visu/index.fcgi?00&session_id={}&lang=en",
                self.config.base_url, *session_id
            )
        };

        debug!("Validating session with test request (session_id: [REDACTED])");

        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    info!("Session is valid");
                    Ok(true)
                } else if response.status() == 401 {
                    warn!("Session is invalid (401)");
                    Ok(false)
                } else {
                    warn!("Session validation returned unexpected status: {}", response.status());
                    Ok(false)
                }
            }
            Err(e) => {
                warn!("Session validation failed: {}", e);
                Ok(false)
            }
        }
    }

    pub async fn ensure_valid_session(&self) -> Result<()> {
        info!("Logging in with credentials from .env...");
        self.refresh_session().await?;
        info!("Login successful!");
        Ok(())
    }

    async fn check_and_refresh_if_unauthorized(&self, response: &reqwest::Response) -> Result<bool> {
        if response.status() == 401 {
            warn!("Got 401 Unauthorized - session expired, refreshing...");
            self.refresh_session().await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn discover_devices(&self) -> Result<Vec<Device>> {
        let mut devices = Vec::new();

        info!("Auto-detecting pages...");
        for page_num in 1..=99 {
            let page = format!("{:02}", page_num);

            info!("Discovering devices on page {}", page);
            let page_devices = self.discover_page_devices(&page).await?;

            if page_devices.is_empty() {
                info!("Page {} is empty, stopping auto-detection", page);
                break;
            }

            info!("Found {} devices on page {}", page_devices.len(), page);
            devices.extend(page_devices);
        }

        info!("Total devices discovered: {}", devices.len());
        Ok(devices)
    }

    async fn discover_page_devices(&self, page: &str) -> Result<Vec<Device>> {
        let url = {
            let session_id = self.session_id.read().await;
            format!(
                "{}/visu/index.fcgi?{}&session_id={}&lang=en",
                self.config.base_url, page, *session_id
            )
        };

        debug!("Fetching page {} (session_id: [REDACTED])", page);
        let response = self.client.get(&url).send().await?;

        if self.check_and_refresh_if_unauthorized(&response).await? {
            let url = {
                let session_id = self.session_id.read().await;
                format!(
                    "{}/visu/index.fcgi?{}&session_id={}&lang=en",
                    self.config.base_url, page, *session_id
                )
            };
            let response = self.client.get(&url).send().await?;
            let html = response.text().await?;
            return self.parse_devices(&html, page);
        }

        let html = response.text().await?;
        self.parse_devices(&html, page)
    }

    fn parse_devices(&self, html: &str, page: &str) -> Result<Vec<Device>> {
        let document = Html::parse_document(html);
        let mut devices = Vec::new();

        let element_selector = Selector::parse(".visu-element").unwrap();
        let name_selector = Selector::parse(".visu-element-name").unwrap();
        let button_selector = Selector::parse(".visu-icon").unwrap();
        let status_selector = Selector::parse(".visu-status-text").unwrap();

        for element in document.select(&element_selector) {
            let id = match element.value().attr("id") {
                Some(id) => id.to_string(),
                None => continue,
            };

            let index = element
                .value()
                .attr("data-index")
                .unwrap_or("")
                .to_string();

            // Extract device name
            let name = element
                .select(&name_selector)
                .next()
                .map(|n| n.text().collect::<String>().trim().to_string())
                .unwrap_or_else(|| id.clone());

            // Skip if name is empty
            if name.is_empty() {
                continue;
            }

            // Determine device type based on classes
            let classes = element.value().attr("class").unwrap_or("");
            let device_type = self.detect_device_type(classes, &name);

            // Skip informational displays
            if name.contains("Datum") || name.contains("Uhrzeit") {
                debug!("Skipping informational device: {}", name);
                continue;
            }

            // Check if device is currently active
            let is_active = element
                .select(&button_selector)
                .next()
                .map(|btn| btn.value().attr("class").unwrap_or("").contains("btn-active"))
                .unwrap_or(false);

            // Extract status text if present
            let status_text = element
                .select(&status_selector)
                .next()
                .map(|s| s.text().collect::<String>().trim().to_string());

            debug!(
                "Found device: id={}, name={}, type={:?}, index={}, active={}, status={:?}",
                id, name, device_type, index, is_active, status_text
            );

            let mut device = Device::new(id, name, device_type, page.to_string(), index);
            device.set_on(is_active);

            devices.push(device);
        }

        Ok(devices)
    }

    fn detect_device_type(&self, classes: &str, name: &str) -> DeviceType {
        let name_lower = name.to_lowercase();

        if name_lower.contains("temperatur") || name_lower.contains("temp.") {
            return DeviceType::TemperatureSensor;
        }

        if classes.contains("visu-slider") {
            return DeviceType::Dimmer;
        }

        if classes.contains("visu-shifter") {
            return DeviceType::WindowCovering;
        }

        if name_lower.contains("szene") {
            return DeviceType::Scene;
        }

        if name_lower.contains("lüftung") {
            return DeviceType::Fan;
        }

        DeviceType::Light
    }

    pub async fn send_command(&self, command: &str) -> Result<()> {
        let session_id = self.session_id.read().await;
        let url = format!(
            "{}/visu/controlKNX?{}&session_id={}",
            self.config.base_url, command, *session_id
        );
        drop(session_id);

        debug!("Sending command: {} (session_id: [REDACTED])", command);
        let response = self.client.post(&url).send().await?;

        if response.status().is_success() {
            debug!("Command sent successfully");
            Ok(())
        } else if response.status() == 401 {
            warn!("Session expired (401), refreshing session...");
            self.refresh_session().await?;
            let session_id = self.session_id.read().await;
            let url = format!(
                "{}/visu/controlKNX?{}&session_id={}",
                self.config.base_url, command, *session_id
            );
            drop(session_id);

            debug!("Retrying command with new session: {}", url);
            let response = self.client.post(&url).send().await?;

            if response.status().is_success() {
                debug!("Command sent successfully after session refresh");
                Ok(())
            } else {
                warn!("Command failed after session refresh: {}", response.status());
                Err(anyhow::anyhow!("Command failed after refresh: {}", response.status()))
            }
        } else {
            warn!("Command failed with status: {}", response.status());
            Err(anyhow::anyhow!("Command failed: {}", response.status()))
        }
    }

    async fn refresh_session(&self) -> Result<()> {
        info!("Refreshing session using headless browser...");

        let username = env::var("SMARTHOME_USERNAME")
            .context("SMARTHOME_USERNAME not set in .env")?;
        let password = env::var("SMARTHOME_PASSWORD")
            .context("SMARTHOME_PASSWORD not set in .env")?;

        info!("Launching headless Chrome...");

        let browser = Browser::new(LaunchOptions {
            headless: false,
            sandbox: false,
            args: vec![
                std::ffi::OsStr::new("--disable-blink-features=AutomationControlled"),
                std::ffi::OsStr::new("--disable-dev-shm-usage"),
                std::ffi::OsStr::new("--disable-web-security"),
                std::ffi::OsStr::new("--user-agent=Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"),
            ],
            ..Default::default()
        })
        .context("Failed to launch Chrome")?;

        let tab = browser.new_tab().context("Failed to create new tab")?;

        let start_url = format!("{}/visu/index.fcgi?00", self.config.base_url);
        info!("Navigating to OAuth login page...");
        tab.navigate_to(&start_url)
            .context("Failed to navigate to start URL")?;

        tab.evaluate(
            "Object.defineProperty(navigator, 'webdriver', {get: () => undefined})",
            false,
        )
        .ok();

        info!("Waiting for login page...");
        tab.wait_for_element_with_custom_timeout("input[name='email']", Duration::from_secs(10))
            .context("Login page email field not found")?;

        info!("Filling email field...");
        let email_element = tab.wait_for_element("input[name='email']")
            .context("Email field not found")?;
        email_element.type_into(&username)
            .context("Failed to fill email")?;

        info!("Filling password field...");
        let password_element = tab.wait_for_element("input[name='password']")
            .context("Password field not found")?;
        password_element.type_into(&password)
            .context("Failed to fill password")?;

        info!("Submitting login form...");
        let submit_button = tab.wait_for_element("button[type='submit']")
            .context("Submit button not found")?;
        submit_button.click()
            .context("Failed to click submit button")?;

        info!("Waiting for redirect to SmartHome...");
        let mut attempts = 0;
        let max_attempts = 20;
        let mut final_url = String::new();

        loop {
            std::thread::sleep(Duration::from_secs(1));
            final_url = tab.get_url();

            if final_url.contains("session_id=") {
                info!("Redirect successful!");
                break;
            }

            attempts += 1;
            if attempts >= max_attempts {
                return Err(anyhow::anyhow!(
                    "Login failed: redirect timeout. Still at: {}",
                    final_url
                ));
            }

            debug!("Waiting for redirect... ({}/{})", attempts, max_attempts);
        }

        info!("OAuth login successful, extracting new session...");

        let new_session_id = self.extract_session_id(&final_url)
            .context("Failed to extract session_id from final URL")?;

        info!("New session ID obtained: [REDACTED]");

        let mut session_id = self.session_id.write().await;
        *session_id = new_session_id.to_string();

        info!("Session ready!");

        Ok(())
    }

    fn extract_session_id(&self, url: &str) -> Result<String> {
        if let Some(session_part) = url.split("session_id=").nth(1) {
            let session_id = session_part
                .split('&')
                .next()
                .unwrap_or(session_part)
                .to_string();

            if session_id.is_empty() {
                return Err(anyhow::anyhow!("session_id is empty in URL"));
            }

            Ok(session_id)
        } else {
            Err(anyhow::anyhow!("No session_id found in URL: {}", url))
        }
    }
}

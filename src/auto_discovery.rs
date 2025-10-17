use anyhow::{Context, Result};
use headless_chrome::{Browser, LaunchOptions};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::time::Duration;
use tracing::{info, warn};
use rand::Rng;

pub struct AutoDiscovery {
    base_url: String,
    username: String,
    password: String,
}

impl AutoDiscovery {
    pub fn new() -> Result<Self> {
        let base_url = env::var("SMARTHOME_BASE_URL")
            .context("SMARTHOME_BASE_URL not set in .env")?;
        let username = env::var("SMARTHOME_USERNAME")
            .context("SMARTHOME_USERNAME not set in .env")?;
        let password = env::var("SMARTHOME_PASSWORD")
            .context("SMARTHOME_PASSWORD not set in .env")?;

        Ok(Self {
            base_url,
            username,
            password,
        })
    }

    // Simulate mouse movement using JavaScript (more compatible)
    fn simulate_mouse_movement(&self, tab: &headless_chrome::Tab, element_selector: &str) -> Result<()> {
        let mut rng = rand::thread_rng();

        // Simulate mouse movement path with JavaScript
        let mouse_js = format!(r#"
            (function() {{
                const element = document.querySelector('{}');
                if (!element) return;

                const rect = element.getBoundingClientRect();
                const targetX = rect.left + (rect.width / 2) + Math.random() * 10 - 5;
                const targetY = rect.top + (rect.height / 2) + Math.random() * 10 - 5;

                // Dispatch mousemove events
                let steps = Math.floor(Math.random() * 10) + 15;
                let startX = Math.random() * 500 + 100;
                let startY = Math.random() * 300 + 100;

                for (let i = 0; i < steps; i++) {{
                    const progress = i / steps;
                    const curve = progress * progress * (3.0 - 2.0 * progress);

                    const x = startX + (targetX - startX) * curve;
                    const y = startY + (targetY - startY) * curve;

                    const event = new MouseEvent('mousemove', {{
                        bubbles: true,
                        cancelable: true,
                        view: window,
                        clientX: x,
                        clientY: y
                    }});
                    document.dispatchEvent(event);
                }}

                // Hover over the element
                const hoverEvent = new MouseEvent('mouseover', {{
                    bubbles: true,
                    cancelable: true,
                    view: window,
                    clientX: targetX,
                    clientY: targetY
                }});
                element.dispatchEvent(hoverEvent);
            }})();
        "#, element_selector);

        tab.evaluate(&mouse_js, false).ok();
        std::thread::sleep(Duration::from_millis(rng.gen_range(200..400)));

        Ok(())
    }

    // Random mouse jitter
    fn random_mouse_jitter(&self, tab: &headless_chrome::Tab) -> Result<()> {
        let mut rng = rand::thread_rng();

        let jitter_js = format!(r#"
            (function() {{
                const x = Math.random() * 1000 + 200;
                const y = Math.random() * 600 + 200;

                for (let i = 0; i < 5; i++) {{
                    const event = new MouseEvent('mousemove', {{
                        bubbles: true,
                        cancelable: true,
                        view: window,
                        clientX: x + Math.random() * 50 - 25,
                        clientY: y + Math.random() * 50 - 25
                    }});
                    document.dispatchEvent(event);
                }}
            }})();
        "#);

        tab.evaluate(&jitter_js, false).ok();
        std::thread::sleep(Duration::from_millis(rng.gen_range(100..300)));

        Ok(())
    }

    // Random scroll
    fn random_scroll(&self, tab: &headless_chrome::Tab) -> Result<()> {
        let mut rng = rand::thread_rng();
        let scroll_amount = rng.gen_range(50..200);

        let scroll_js = format!("window.scrollBy({{ top: {}, behavior: 'smooth' }});", scroll_amount);
        tab.evaluate(&scroll_js, false).ok();

        std::thread::sleep(Duration::from_millis(rng.gen_range(300..600)));

        Ok(())
    }

    pub async fn discover_all_mappings(&self, _pages: &[String]) -> Result<HashMap<String, String>> {
        info!("🔍 Starting auto-discovery mode...");
        info!("Auto-detecting all pages with devices...");
        info!("");
        info!("📋 How this works:");
        info!("   1. Chrome will open and attempt automatic login");
        info!("   2. If CAPTCHA appears, YOU need to solve it manually");
        info!("   3. Session will be saved to chrome_data/");
        info!("   4. Next runs will skip login entirely!");
        info!("");

        let mut all_mappings = HashMap::new();

        info!("Launching Chrome...");

        // Check if user wants to use system Chrome profile
        let use_system_profile = env::var("USE_SYSTEM_CHROME").is_ok();

        let chrome_data = if use_system_profile {
            // Try to use the system's default Chrome profile for better stealth
            let system_profile = if cfg!(target_os = "windows") {
                // Windows: C:\Users\USERNAME\AppData\Local\Google\Chrome\User Data
                let username = env::var("USERNAME").unwrap_or_else(|_| "Administrator".to_string());
                std::path::PathBuf::from(format!(
                    "C:\\Users\\{}\\AppData\\Local\\Google\\Chrome\\User Data",
                    username
                ))
            } else if cfg!(target_os = "macos") {
                // macOS: ~/Library/Application Support/Google/Chrome
                let home = env::var("HOME").unwrap_or_else(|_| "/Users".to_string());
                std::path::PathBuf::from(format!(
                    "{}/Library/Application Support/Google/Chrome",
                    home
                ))
            } else {
                // Linux: ~/.config/google-chrome
                let home = env::var("HOME").unwrap_or_else(|_| "/home".to_string());
                std::path::PathBuf::from(format!("{}/.config/google-chrome", home))
            };

            if system_profile.exists() {
                info!("✅ Using your real Chrome profile (looks more human!)");
                system_profile
            } else {
                info!("⚠️  System Chrome profile not found, using local chrome_data/");
                let local_data = std::env::current_dir()?.join("chrome_data");
                std::fs::create_dir_all(&local_data)?;
                local_data
            }
        } else {
            // Use local profile by default (more reliable)
            info!("Using dedicated chrome_data/ profile (set USE_SYSTEM_CHROME=1 to use real profile)");
            let local_data = std::env::current_dir()?.join("chrome_data");
            std::fs::create_dir_all(&local_data)?;
            local_data
        };

        let browser = Browser::new(LaunchOptions {
            headless: false,
            sandbox: false,
            user_data_dir: Some(chrome_data),
            window_size: Some((1920, 1080)),
            args: vec![
                // Hide automation indicators
                std::ffi::OsStr::new("--disable-blink-features=AutomationControlled"),
                std::ffi::OsStr::new("--exclude-switches=enable-automation"),
                std::ffi::OsStr::new("--disable-infobars"),
                std::ffi::OsStr::new("--disable-extensions"),
                std::ffi::OsStr::new("--no-first-run"),
                std::ffi::OsStr::new("--no-default-browser-check"),
                std::ffi::OsStr::new("--disable-popup-blocking"),
                // Performance & stealth
                std::ffi::OsStr::new("--disable-dev-shm-usage"),
                std::ffi::OsStr::new("--disable-web-security"),
                std::ffi::OsStr::new("--disable-features=IsolateOrigins,site-per-process"),
                std::ffi::OsStr::new("--disable-site-isolation-trials"),
                std::ffi::OsStr::new("--start-maximized"),
                std::ffi::OsStr::new("--disable-setuid-sandbox"),
                std::ffi::OsStr::new("--user-agent=Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36"),
            ],
            ..Default::default()
        })
        .context("Failed to launch Chrome")?;

        let tab = browser.new_tab().context("Failed to create tab")?;

        self.login(&tab)?;

        let mut consecutive_empty_pages = 0;

        for page_num in 1..=99 {
            let page = format!("{:02}", page_num);
            info!("📄 Discovering devices on page {}...", page);
            let page_mappings = self.discover_page(&tab, &page)?;

            if page_mappings.is_empty() {
                consecutive_empty_pages += 1;
                info!("Page {} is empty ({} consecutive empty pages)", page, consecutive_empty_pages);

                if consecutive_empty_pages >= 2 {
                    info!("Found 2 consecutive empty pages, stopping auto-detection");
                    break;
                }
            } else {
                consecutive_empty_pages = 0;
                all_mappings.extend(page_mappings);
            }

            std::thread::sleep(Duration::from_millis(500));
        }

        info!("✅ Discovery complete! Found {} device mappings", all_mappings.len());

        self.save_mappings(&all_mappings)?;

        Ok(all_mappings)
    }

    // Check if already logged in by looking for logout button or main UI
    fn is_logged_in(&self, tab: &headless_chrome::Tab) -> bool {
        // Check if we're already on the main page (not login page)
        let check_js = r#"
            (function() {
                // Check if login form exists
                const hasLoginForm = !!document.querySelector('input[name="email"]');
                // Check if we're on the main visu page
                const hasVisuElements = !!document.querySelector('[data-index]') ||
                                       !!document.querySelector('.visu-icon') ||
                                       window.location.pathname.includes('/visu/');

                return !hasLoginForm && hasVisuElements;
            })();
        "#;

        tab.evaluate(check_js, false)
            .ok()
            .and_then(|result| result.value)
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    fn login(&self, tab: &headless_chrome::Tab) -> Result<()> {
        info!("Checking login status...");

        let start_url = format!("{}/visu/index.fcgi?00", self.base_url);
        tab.navigate_to(&start_url)
            .context("Failed to navigate to start URL")?;

        std::thread::sleep(Duration::from_secs(3));

        // Check if already logged in
        if self.is_logged_in(tab) {
            info!("✅ Already logged in! (Session restored from chrome_data/)");
            return Ok(());
        }

        info!("Not logged in. Attempting automatic login...");

        // Comprehensive anti-detection JavaScript
        let stealth_js = r#"
            // Remove webdriver property
            Object.defineProperty(navigator, 'webdriver', {
                get: () => false,
                configurable: true
            });

            // Remove automation flags
            delete navigator.__webdriver_script_fn;
            delete navigator.__driver_evaluate;
            delete navigator.__webdriver_evaluate;
            delete navigator.__selenium_evaluate;
            delete navigator.__fxdriver_evaluate;
            delete navigator.__driver_unwrapped;
            delete navigator.__webdriver_unwrapped;
            delete navigator.__selenium_unwrapped;
            delete navigator.__fxdriver_unwrapped;

            // Overwrite the `plugins` property
            Object.defineProperty(navigator, 'plugins', {
                get: () => [1, 2, 3, 4, 5]
            });

            // Overwrite the `languages` property
            Object.defineProperty(navigator, 'languages', {
                get: () => ['en-US', 'en']
            });

            // Add chrome object
            if (!window.chrome) {
                window.chrome = {};
            }
            window.chrome.runtime = {
                connect: () => {},
                sendMessage: () => {}
            };

            // Mock permissions
            const originalQuery = window.navigator.permissions.query;
            window.navigator.permissions.query = (parameters) => (
                parameters.name === 'notifications' ?
                    Promise.resolve({ state: Notification.permission }) :
                    originalQuery(parameters)
            );

            // Hide automation in toString
            const originalToString = Function.prototype.toString;
            Function.prototype.toString = function() {
                if (this === window.navigator.permissions.query) {
                    return 'function query() { [native code] }';
                }
                return originalToString.call(this);
            };

            // Remove CDP detection
            delete Object.getPrototypeOf(navigator).webdriver;

            // Spoof connection type
            Object.defineProperty(navigator, 'connection', {
                get: () => ({
                    effectiveType: '4g',
                    rtt: 50,
                    downlink: 10,
                    saveData: false
                })
            });

            // Add realistic platform info
            Object.defineProperty(navigator, 'platform', {
                get: () => 'MacIntel'
            });

            // Mock hardware concurrency
            Object.defineProperty(navigator, 'hardwareConcurrency', {
                get: () => 8
            });

            // Mock device memory
            Object.defineProperty(navigator, 'deviceMemory', {
                get: () => 8
            });
        "#;

        tab.evaluate(stealth_js, false).ok();

        // Add delay to seem human
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Random mouse movement after page load
        self.random_mouse_jitter(tab).ok();
        std::thread::sleep(Duration::from_millis(500));

        // Maybe scroll a bit
        self.random_scroll(tab).ok();
        std::thread::sleep(Duration::from_millis(800));

        tab.wait_for_element_with_custom_timeout("input[name='email']", Duration::from_secs(10))
            .context("Login page not found")?;

        // Simulate mouse movement to email field
        self.simulate_mouse_movement(tab, "input[name='email']")?;
        std::thread::sleep(Duration::from_millis(300));

        // Click and type
        let email_element = tab.wait_for_element("input[name='email']")?;
        email_element.click()?;
        std::thread::sleep(Duration::from_millis(200));
        email_element.type_into(&self.username)?;

        // Move to password field
        std::thread::sleep(Duration::from_millis(800));
        self.simulate_mouse_movement(tab, "input[name='password']")?;
        std::thread::sleep(Duration::from_millis(250));

        let password_element = tab.wait_for_element("input[name='password']")?;
        password_element.click()?;
        std::thread::sleep(Duration::from_millis(200));
        password_element.type_into(&self.password)?;

        // Move to submit button
        std::thread::sleep(Duration::from_millis(700));
        self.simulate_mouse_movement(tab, "button[type='submit']")?;
        std::thread::sleep(Duration::from_millis(400));

        // Try to find and click submit button
        match tab.wait_for_element("button[type='submit']") {
            Ok(submit_button) => {
                submit_button.click().ok();
                info!("Submit button clicked, waiting for response...");
            }
            Err(_) => {
                info!("⚠️  Could not find submit button automatically");
            }
        }

        // Wait and check if login succeeded
        std::thread::sleep(Duration::from_secs(5));

        // Check if we're logged in now
        if self.is_logged_in(tab) {
            info!("✅ Automatic login successful!");
            return Ok(());
        }

        // If automatic login failed, wait for manual intervention
        info!("⚠️  Automatic login failed or CAPTCHA detected");
        info!("👤 Please complete the login manually in the Chrome window");
        info!("   (Solve CAPTCHA if needed, then submit the form)");
        info!("   Waiting up to 2 minutes for manual login...");

        // Poll for successful login (120 seconds = 2 minutes)
        let mut attempts = 0;
        let max_attempts = 120;

        while attempts < max_attempts {
            std::thread::sleep(Duration::from_secs(1));
            attempts += 1;

            if self.is_logged_in(tab) {
                info!("✅ Manual login successful!");
                info!("   Session saved to chrome_data/ - next run will skip login!");
                return Ok(());
            }

            // Show progress every 15 seconds
            if attempts % 15 == 0 {
                info!("   Still waiting... ({}/{} seconds)", attempts, max_attempts);
            }
        }

        anyhow::bail!("Login timeout: Please try again")
    }

    fn discover_page(&self, tab: &headless_chrome::Tab, page: &str) -> Result<HashMap<String, String>> {
        let mut mappings = HashMap::new();

        let page_url = format!("{}/visu/index.fcgi?{}", self.base_url, page);
        tab.navigate_to(&page_url)?;

        std::thread::sleep(Duration::from_secs(3));

        let count_script = "document.querySelectorAll('[data-index][data-page]').length";
        let count_result = tab.evaluate(count_script, false)?;
        info!("  Found {} elements with data-index and data-page", count_result.value.as_ref().unwrap_or(&serde_json::Value::Number(0.into())));
        let script = r#"
            JSON.stringify(
                Array.from(document.querySelectorAll('[data-index][data-page]')).map(function(el) {
                    const iconButton = el.querySelector('.visu-icon');
                    const iconClass = iconButton ? iconButton.className : '';

                    return {
                        id: el.id,
                        name: el.textContent.trim(),
                        index: el.getAttribute('data-index'),
                        page: el.getAttribute('data-page'),
                        isShifter: el.classList.contains('visu-shifter'),
                        className: el.className,
                        iconClass: iconClass
                    };
                })
            )
        "#;

        info!("  Extracting device information from HTML...");
        let elements = tab.evaluate(script, false)?;

        info!("  JavaScript result type: {:?}", elements.value.as_ref().map(|v| v.to_string().chars().take(200).collect::<String>()));

        let array: Vec<serde_json::Value> = if let Some(json_str) = elements.value.as_ref().and_then(|v| v.as_str()) {
            serde_json::from_str(json_str).unwrap_or_default()
        } else {
            Vec::new()
        };

        info!("  Found {} devices on page {}", array.len(), page);

        if !array.is_empty() {

            for element in array.iter() {
                if let Some(obj) = element.as_object() {
                    let id = obj.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let index = obj.get("index").and_then(|v| v.as_str()).unwrap_or("");
                    let device_page = obj.get("page").and_then(|v| v.as_str()).unwrap_or("");
                    let is_shifter = obj.get("isShifter").and_then(|v| v.as_bool()).unwrap_or(false);
                    let icon_class = obj.get("iconClass").and_then(|v| v.as_str()).unwrap_or("");

                    if id.is_empty() || index.is_empty() {
                        continue;
                    }

                    let icon_type = icon_class.split_whitespace()
                        .find(|s| s.starts_with("icon-"))
                        .unwrap_or("");

                    if is_shifter {
                        let device_key = format!("{}_page{}", id, device_page);

                        let cmd_up = format!("{}+01+00+{}", index, device_page);
                        let cmd_stop = format!("{}+02+00+{}", index, device_page);
                        let cmd_down = format!("{}+03+00+{}", index, device_page);

                        mappings.insert(format!("{}_up", device_key), cmd_up.clone());
                        mappings.insert(format!("{}_stop", device_key), cmd_stop.clone());
                        mappings.insert(format!("{}_down", device_key), cmd_down.clone());

                        info!("    ✓ {} (Blind) → UP: {}, STOP: {}, DOWN: {}",
                            name, cmd_up, cmd_stop, cmd_down);
                    } else {
                        let command = format!("{}+01+00+{}", index, device_page);
                        let device_key = format!("{}_page{}", id, device_page);

                        mappings.insert(format!("{}_{}", device_key, icon_type), command.clone());
                        info!("    ✓ {} → {}", name, command);
                    }
                }
            }
        }

        Ok(mappings)
    }

    fn save_mappings(&self, mappings: &HashMap<String, String>) -> Result<()> {
        info!("💾 Saving mappings to device_mappings_auto.toml...");

        let mut lights = HashMap::new();
        let mut blinds = HashMap::new();
        let mut dimmers = HashMap::new();
        let mut ventilation = HashMap::new();
        let mut scenes = HashMap::new();
        let mut sensors = HashMap::new();
        let mut switches = HashMap::new();

        for (key, command) in mappings {
            let clean_key = key.split("_icon-").next().unwrap_or(&key).to_string();

            if key.contains("Double3") {
                blinds.insert(clean_key, command.clone());
            } else if key.contains("ExtendedSlider") {
                dimmers.insert(clean_key, command.clone());
            } else if key.contains("icon-45") {
                ventilation.insert(clean_key, command.clone());
            } else if key.contains("Szene") || key.contains("Scene") || key.contains("icon-11") || key.contains("icon-76") {
                scenes.insert(clean_key, command.clone());
            } else if key.contains("Temp") || key.contains("Datum") || key.contains("Uhrzeit") || key.contains("gesperrt") {
                sensors.insert(clean_key, command.clone());
            } else if key.contains("Single") {
                lights.insert(clean_key, command.clone());
            } else {
                switches.insert(clean_key, command.clone());
            }
        }

        let mut content = String::new();
        content.push_str("# Auto-generated device mappings\n");
        content.push_str("# Generated by auto-discovery mode\n\n");

        if !lights.is_empty() {
            content.push_str("[lights]\n");
            for (key, cmd) in lights {
                content.push_str(&format!("\"{}\" = \"{}\"\n", key, cmd));
            }
            content.push_str("\n");
        }

        if !blinds.is_empty() {
            content.push_str("[blinds]\n");
            for (key, cmd) in blinds {
                content.push_str(&format!("\"{}\" = \"{}\"\n", key, cmd));
            }
            content.push_str("\n");
        }

        if !dimmers.is_empty() {
            content.push_str("[dimmers]\n");
            for (key, cmd) in dimmers {
                content.push_str(&format!("\"{}\" = \"{}\"\n", key, cmd));
            }
            content.push_str("\n");
        }

        if !ventilation.is_empty() {
            content.push_str("[ventilation]\n");
            for (key, cmd) in ventilation {
                content.push_str(&format!("\"{}\" = \"{}\"\n", key, cmd));
            }
            content.push_str("\n");
        }

        if !scenes.is_empty() {
            content.push_str("[scenes]\n");
            for (key, cmd) in scenes {
                content.push_str(&format!("\"{}\" = \"{}\"\n", key, cmd));
            }
            content.push_str("\n");
        }

        if !sensors.is_empty() {
            content.push_str("[sensors]\n");
            for (key, cmd) in sensors {
                content.push_str(&format!("\"{}\" = \"READONLY\"\n", key));
            }
            content.push_str("\n");
        }

        if !switches.is_empty() {
            content.push_str("[switches]\n");
            for (key, cmd) in switches {
                content.push_str(&format!("\"{}\" = \"{}\"\n", key, cmd));
            }
            content.push_str("\n");
        }

        fs::write("device_mappings_auto.toml", content)
            .context("Failed to write device_mappings_auto.toml")?;

        info!("✅ Saved to device_mappings_auto.toml");
        info!("You can review it and rename to device_mappings.toml");

        Ok(())
    }
}

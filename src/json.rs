//! This module uses a token.json file in a resources folder inside your
//! project's base directory.
//!
//! The token.json file should only contain the "dev_id" and "token" fields.

use chrono::{DateTime, Utc};
use md5;
use reqwest;
use serde_json::Value;
use std::fs::File;
use std::io::prelude::*;

use super::response::Session;

const BASE_LINK: &str = "http://api.smitegame.com/smiteapi.svc";
const SECRET_PATH: &str = "resources/token.json";

/// Read a given file into a String and return the result.
pub fn read_file_to_string(path: &str) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;

    Ok(file_contents)
}

/// Create/Write a String to a given path.
pub fn write_string_to_file(path: &str, data: String) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(&mut data.as_bytes())?;
    Ok(())
}

/// Use this to read the dev id and auth key from the token.json file.
fn read_secret(secret_key: &str) -> String {
    let token_file = read_file_to_string(SECRET_PATH).unwrap();
    let json: Value = serde_json::from_str(&token_file).unwrap();

    json[secret_key].as_str().unwrap().to_string()
}

/// This returns time in YYYYMMDDHHSS format.
/// Required for API queries.
pub fn get_formatted_time() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.format("%Y%m%d%H%M%S").to_string()
}

/// Generates an MD5-Hashed signature required for API queries.
fn make_signature(dev_id: &str, methodname: &str, token: &str, time: &str) -> String {
    let unhashed_signature = format!("{}{}{}{}", dev_id, methodname, token, time);
    let bytes = unhashed_signature.as_bytes();
    format!("{:x}", md5::compute(bytes))
}

/// Use signature to create the session link.
fn create_session_link() -> String {
    let dev_id = read_secret("dev_id");
    let token = read_secret("token");
    let method = "createsession";
    let time = get_formatted_time();
    let signature = make_signature(&dev_id, method, &token, &time);

    format!(
        "{}/{}json/{}/{}/{}",
        BASE_LINK, method, dev_id, signature, time
    )
}

/// Use session id to create links to any method call.
/// NOTE: The timestamp here refers to what time you want the data from,
/// it is not the "time" from the signature.
pub fn create_link(method: &str, session_id: &str, timestamp: &str) -> String {
    let dev_id = read_secret("dev_id");
    let time = get_formatted_time();
    let token = read_secret("token");
    let signature = make_signature(&dev_id, method, &token, &time);

    format!(
        "{}/{}json/{}/{}/{}/{}",
        BASE_LINK, method, dev_id, signature, session_id, timestamp
    )
}

/// Make a request using a link and return the json from the API query.
pub fn fetch_json(link: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::blocking::get(link)?.text()?;
    Ok(response)
}

/// Create the session and return the session object.
/// The object contains the session id and timestamp.
/// This is REQUIRED to use the API and only lasts 15 minutes.
pub fn make_session() -> Result<Session, reqwest::Error> {
    let link = create_session_link();
    let response = fetch_json(&link)?;
    let mut session: Session = serde_json::from_str(&response).unwrap();

    // Use our formatted time for timestamp instead. Offset of 15 sec just to be safe.
    session.timestamp = (get_formatted_time().parse::<usize>().unwrap() - 15).to_string();
    Ok(session)
}

/*
   Example for request data from Fritz!Box via AHA interface

   No Error handling provided
*/

// TODO check if session is still valid
// TODO comment code // add docu
// TODO add tests
// TODO error handling
// TODO add resources
// TODO add logging

use pbkdf2::{pbkdf2_hmac, pbkdf2_hmac_array};
use serde::Deserialize;
use sha2::Sha256;
use std::env;

fn main() {
    let fritz = get_data();
    let sid = fritz.session_id.unwrap_or_else(|| {
        let challenge = request_challange(&fritz.url);
        let response = solve_pbkdf2_challenge(&fritz.password, &challenge);
        login(&fritz.url, &fritz.user, &response)
    });
    if sid != "0000000000000000" {
        env::set_var("FRITZ_SESSION_ID", &sid);
    }
    println!("sid: {}", sid);
    let result = execute_command(&fritz.url, &sid);
    println!("result: {}", result);
}

#[derive(Debug)]
struct FritzBox {
    url: String,
    user: String,
    password: String,
    session_id: Option<String>,
}

fn get_data() -> FritzBox {
    let url = env::var("FRITZ_URL").expect("FRITZ_URL not set");
    let user = env::var("FRITZ_USER").expect("FRITZ_USER not set");
    let password = env::var("FRITZ_PASSWORD").unwrap_or_else(|_| ask_password());
    let session_id = env::var("FRITZ_SESSION_ID").ok();

    FritzBox {
        url,
        user,
        password,
        session_id,
    }
}

fn ask_password() -> String {
    rpassword::prompt_password("Enter Fritz!Box user password or set it via ENV: ").unwrap()
}

#[derive(Debug, Deserialize)]
struct SessionInfo {
    #[serde(rename = "SID")]
    sid: String,
    #[serde(rename = "Challenge")]
    challenge: String,
    #[serde(rename = "BlockTime")]
    block_time: u32,
}

fn request_challange(url: &str) -> String {
    let url = format!("{}/login_sid.lua?version=2", url);
    let body = reqwest::blocking::get(url).unwrap().text().unwrap();
    println!("body: {}", body);
    let session_info: SessionInfo = serde_xml_rs::from_str(&body).unwrap();
    if session_info.block_time > 0 {
        panic!(
            "block_time: {}, you have to wait until you can login again",
            session_info.block_time
        );
    }
    println!("session_info: {:?}", session_info);
    session_info.challenge
}

fn solve_pbkdf2_challenge(password: &str, challenge: &str) -> String {
    let mut challenge_parts = challenge.split('$');
    let version = challenge_parts.next().unwrap();
    assert!(version == "2");
    let iter1: u32 = challenge_parts.next().unwrap().parse().unwrap();
    let salt1 = hex::decode(challenge_parts.next().unwrap()).unwrap();
    let iter2: u32 = challenge_parts.next().unwrap().parse().unwrap();
    let salt2 = hex::decode(challenge_parts.next().unwrap()).unwrap();

    let mut hash1 = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt1, iter1, &mut hash1);
    let key = pbkdf2_hmac_array::<Sha256, 32>(&hash1, &salt2, iter2);
    let hex_key = hex::encode(key);
    let response = format!("{}${hex_key}", hex::encode_upper(salt2));

    response
}

fn login(url: &str, user: &str, response: &str) -> String {
    let url = format!("{}/login_sid.lua?version=2", url);
    let url = reqwest::Url::parse_with_params(&url, &[("username", user), ("response", response)])
        .unwrap();
    let body = reqwest::blocking::get(url).unwrap().text().unwrap();
    println!("body: {}", body);
    let session_info: SessionInfo = serde_xml_rs::from_str(&body).unwrap();
    println!("session_info: {:?}", session_info);
    session_info.sid
}

fn execute_command(url: &str, sid: &str) -> String {
    let url = format!("{}//webservices/homeautoswitch.lua", url);
    let url = reqwest::Url::parse_with_params(
        &url,
        &[
            ("sid", sid),
            ("switchcmd", "getswitchpower"),
            ("ain", "116300016812"),
        ],
    )
    .unwrap();
    let body = reqwest::blocking::get(url).unwrap().text().unwrap();
    println!("body: {}", body);
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pbkdf2_challenge() {
        let password = "1example!";
        let challenge = "2$10000$5A1711$2000$5A1722";

        let response = solve_pbkdf2_challenge(password, challenge);
        assert_eq!(
            response,
            "5A1722$1798a1672bca7c6463d6b245f82b53703b0f50813401b03e4045a5861e689adb"
        )
    }
}

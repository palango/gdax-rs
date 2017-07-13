extern crate reqwest;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;

use std::fmt::Display;
use std::str::FromStr;

use serde::de::DeserializeOwned;
use serde_json::de as de_json;
use serde::de::{self, Deserialize, Deserializer};

const PUBLIC_API_URL: &'static str = "https://api.gdax.com";

fn create_api_url(path: &str) -> String {
    format!("{}/{}", PUBLIC_API_URL, path)
}

#[derive(Debug)]
pub enum Error {
    Api,
    Http(reqwest::Error),
//    InvalidSecretKey,
    Json(serde_json::Error),
}

impl std::convert::From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::Http(err)
    }
}

impl std::convert::From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::Json(err)
    }
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Product {
    pub id: String,
    pub base_currency: String,
    pub quote_currency: String,
    #[serde(deserialize_with = "from_str")]
    pub base_min_size: f64,
    #[serde(deserialize_with = "from_str")]
    pub base_max_size: f64,
    #[serde(deserialize_with = "from_str")]
    pub quote_increment: f64,
    pub display_name: String,
}

fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where T: FromStr,
          T::Err: Display,
          D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}

pub struct Client {
    http_client: reqwest::Client,
}

impl Client {
    pub fn new() -> Client {
        Client {
            http_client: reqwest::Client::new().expect("could not create HTTP client."),
        }
    }

    fn get_and_decode<T>(&self, url: &str) -> Result<T, Error>
        where T: DeserializeOwned
    {
        let mut res = self.http_client.get(url)?.send()?;

        if !res.status().is_success() {
            return Err(Error::Api);
        }

        Ok(de_json::from_reader(&mut res)?)
    }

    pub fn get_products(&self) -> Result<Vec<Product>, Error> {
        self.get_and_decode(&create_api_url("products"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let c = Client::new();
        let res = c.get_products();
        println!("res = {:#?}", res);
        assert!(res.is_ok());
    }
}

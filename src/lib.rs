extern crate reqwest;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate uuid;

use std::fmt::Display;
use std::str::FromStr;

use serde::de::DeserializeOwned;
use serde_json::de as de_json;
use serde::de::{self, Deserialize, Deserializer};


use uuid::Uuid;

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

#[derive(Deserialize, Debug)]
pub struct BookEntry {
    #[serde(deserialize_with = "from_str")]
    pub price: f64,
    #[serde(deserialize_with = "from_str")]
    pub size: f64,
    pub num_orders: u64
}

#[derive(Deserialize, Debug)]
pub struct OrderBook<T> {
    pub sequence: usize,
    pub bids: Vec<T>,
    pub asks: Vec<T>
}

fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where T: FromStr,
          T::Err: Display,
          D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}

pub enum Level {
    Best    = 1,
    Top50   = 2,
    Full    = 3
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
        println!("url = {:#?}", url);
        let mut res = self.http_client.get(url)?.send()?;

        if !res.status().is_success() {
            return Err(Error::Api);
        }

        Ok(de_json::from_reader(&mut res)?)
    }

    pub fn get_products(&self) -> Result<Vec<Product>, Error> {
        self.get_and_decode(&create_api_url("products"))
    }

    pub fn get_best_order(&self, product: &str) -> Result<OrderBook<BookEntry>, Error> {
        self.get_and_decode(&format!("{}/products/{}/book?level={}",
                                     PUBLIC_API_URL,
                                     product,
                                     Level::Best as u8))
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


    #[test]
    fn it_works2() {
        let c = Client::new();
        let res = c.get_best_order("ETH-USD");
        println!("res = {:#?}", res);
        assert!(res.is_ok());
    }
}

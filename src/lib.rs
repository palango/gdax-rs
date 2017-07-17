extern crate reqwest;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate uuid;
extern crate chrono;

use std::fmt::{self, Display};
use std::str::FromStr;

use chrono::{DateTime, Utc, Duration};
use serde::de::DeserializeOwned;
use serde_json::de as de_json;
use serde::de::{self, Deserialize, Deserializer};


use uuid::Uuid;

const PUBLIC_API_URL: &'static str = "https://api.gdax.com";

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
pub struct FullBookEntry {
    #[serde(deserialize_with = "from_str")]
    pub price: f64,
    #[serde(deserialize_with = "from_str")]
    pub size: f64,
    #[serde(deserialize_with = "from_str")]
    pub order_id: Uuid
}

#[derive(Deserialize, Debug)]
pub struct OrderBook<T> {
    pub sequence: usize,
    pub bids: Vec<T>,
    pub asks: Vec<T>
}

#[derive(Deserialize, Debug)]
pub struct Candle {
    pub time: u64,
    pub low: f64,
    pub high: f64,
    pub open: f64,
    pub close: f64,
    pub volume: f64
}

#[derive(Deserialize, Debug)]
pub struct Tick {
    pub trade_id: u64,
    #[serde(deserialize_with = "from_str")]
    pub price: f64,
    #[serde(deserialize_with = "from_str")]
    pub size: f64,
    #[serde(deserialize_with = "from_str")]
    pub bid: f64,
    #[serde(deserialize_with = "from_str")]
    pub ask: f64,
    #[serde(deserialize_with = "from_str")]
    pub volume: f64,
    pub time: DateTime<Utc>
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Side {
    Buy,
    Sell
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Side::Buy => write!(f, "Buy"),
            Side::Sell => write!(f, "Sell")
        }
    }
}

// We manually implement Serialize for Side here
// because the default encoding/decoding scheme that derive
// gives us isn't the straightforward mapping unfortunately
impl serde::Serialize for Side {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        match *self {
            Side::Buy => serializer.serialize_str("buy"),
            Side::Sell => serializer.serialize_str("sell")
        }
    }
}

// We manually implement Deserialize for Side here
// because the default encoding/decoding scheme that derive
// gives us isn't the straightforward mapping unfortunately
impl<'de> serde::Deserialize<'de> for Side {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de>
    {
        struct SideVisitor;

        impl<'de> serde::de::Visitor<'de> for SideVisitor {
            type Value = Side;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a Side specifier, either Buy or Sell")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where E: serde::de::Error {
                match &*v.to_lowercase() {
                    "buy" => Ok(Side::Buy),
                    "sell" => Ok(Side::Sell),
                    //                    _ => Err(E::invalid_value(serde::de::Unexpected::Str(v), &self)),
                    _ => Err(E::custom(format!("side must be either `buy` or `sell`: {}", v))),
                }
            }
        }
        deserializer.deserialize_str(SideVisitor)
    }
}

#[derive(Deserialize, Debug)]
pub struct Trade {
    pub time: DateTime<Utc>,
    pub trade_id: u64,
    #[serde(deserialize_with = "from_str")]
    pub price: f64,
    #[serde(deserialize_with = "from_str")]
    pub size: f64,
    pub side: Side,
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
    Best = 1,
    Top50 = 2,
    Full = 3
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
        self.get_and_decode(&format!("{}/products", PUBLIC_API_URL))
    }

    pub fn get_best_order(&self, product: &str) -> Result<OrderBook<BookEntry>, Error> {
        self.get_and_decode(&format!("{}/products/{}/book?level={}",
                                     PUBLIC_API_URL,
                                     product,
                                     Level::Best as u8))
    }

    pub fn get_top50_orders(&self, product: &str) -> Result<OrderBook<BookEntry>, Error> {
        self.get_and_decode(&format!("{}/products/{}/book?level={}",
                                     PUBLIC_API_URL,
                                     product,
                                     Level::Top50 as u8))
    }

    pub fn get_full_book(&self, product: &str) -> Result<OrderBook<FullBookEntry>, Error> {
        self.get_and_decode(&format!("{}/products/{}/book?level={}",
                                     PUBLIC_API_URL,
                                     product,
                                     Level::Full as u8))
    }

    pub fn get_historic_rates(&self,
                              product: &str,
                              start_time: DateTime<Utc>,
                              end_time: DateTime<Utc>,
                              granularity: Duration)
                              -> Result<Vec<Candle>, Error> {
        self.get_and_decode(&format!("{}/products/{}/candles?start={}&end={}&granularity={}",
                                     PUBLIC_API_URL,
                                     product,
                                     start_time.to_rfc3339(),
                                     end_time.to_rfc3339(),
                                     granularity.num_seconds()))
    }

    pub fn get_product_ticker(&self, product: &str) -> Result<Tick, Error> {
        self.get_and_decode(&format!("{}/products/{}/ticker", PUBLIC_API_URL, product))
    }

    pub fn get_trades(&self, product: &str) -> Result<Vec<Trade>, Error> {
        self.get_and_decode(&format!("{}/products/{}/trades", PUBLIC_API_URL, product))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    //    #[test]
    //    fn it_works() {
    //        let c = Client::new();
    //        let res = c.get_products();
    //        println!("res = {:#?}", res);
    //        assert!(res.is_ok());
    //    }
    //
    //
    //    #[test]
    //    fn it_works2() {
    //        let c = Client::new();
    //        let res = c.get_best_order("ETH-USD");
    //        println!("res = {:#?}", res);
    //        assert!(res.is_ok());
    //    }
    //    #[test]
    //    fn it_works3() {
    //        let c = Client::new();
    //        let res = c.get_top50_orders("ETH-USD");
    //        println!("res = {:#?}", res);
    //        assert!(res.is_ok());
    //    }
    //        #[test]
    //        fn it_works4() {
    //            let c = Client::new();
    //            let res = c.get_full_book("ETH-USD");
    //            println!("res = {:#?}", res);
    //            assert!(res.is_ok());
    //        }
    //    #[test]
    //    fn it_works4() {
    //        let c = Client::new();
    //        let now = Utc::now();
    //        let diff = Duration::seconds(200);
    //        let then = now - diff;
    //        let res = c.get_historic_rates("ETH-USD", now, then, Duration::seconds(1));
    //        println!("res = {:#?}", res);
    //        assert!(res.is_ok());
    //    }


//    #[test]
//    fn it_works5() {
//        let c = Client::new();
//        let res = c.get_trades("ETH-USD");
//        println!("res = {:#?}", res);
//        assert!(res.is_ok());
//    }

    #[test]
    fn it_works6() {
        let c = Client::new();
        let res = c.get_product_ticker("ETH-USD");
        println!("res = {:#?}", res);
        assert!(res.is_ok());
    }
}

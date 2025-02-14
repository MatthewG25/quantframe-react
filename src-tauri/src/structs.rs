use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum GlobleError {
    ReqwestError(reqwest::Error),
    SerdeError(String, usize, usize),
    PolarsError(String),
    IoError(std::io::Error),
    ParseIntError(std::num::ParseIntError),
    ParseFloatError(std::num::ParseFloatError),
    ParseBoolError(std::str::ParseBoolError),
    DabaseError(sqlx::Error),
    ParseError(String),
    TooManyRequests(String),
    OtherError(String),
    HttpError(reqwest::StatusCode, String, String),
}
impl serde::Serialize for GlobleError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        return GlobleError::serialize(self, serializer);
    }
}

impl<T> From<std::sync::PoisonError<T>> for GlobleError {
    fn from(e: std::sync::PoisonError<T>) -> Self {
        GlobleError::OtherError(format!("{:?}", e))
    }
}
impl From<sqlx::Error> for GlobleError {
    fn from(e: sqlx::Error) -> Self {
        GlobleError::DabaseError(e)
    }
}

impl From<reqwest::Error> for GlobleError {
    fn from(e: reqwest::Error) -> Self {
        GlobleError::ReqwestError(e)
    }
}
impl From<serde_json::Error> for GlobleError {
    fn from(e: serde_json::Error) -> Self {
        GlobleError::SerdeError(format!("{:?}", e), e.line(), e.column())
    }
}
impl From<polars::error::PolarsError> for GlobleError {
    fn from(e: polars::error::PolarsError) -> Self {
        GlobleError::PolarsError(format!("{:?}", e))
    }
}
impl From<std::io::Error> for GlobleError {
    fn from(e: std::io::Error) -> Self {
        GlobleError::IoError(e)
    }
}
impl From<std::num::ParseIntError> for GlobleError {
    fn from(e: std::num::ParseIntError) -> Self {
        GlobleError::ParseIntError(e)
    }
}
impl From<std::num::ParseFloatError> for GlobleError {
    fn from(e: std::num::ParseFloatError) -> Self {
        GlobleError::ParseFloatError(e)
    }
}
impl From<std::str::ParseBoolError> for GlobleError {
    fn from(e: std::str::ParseBoolError) -> Self {
        GlobleError::ParseBoolError(e)
    }
}
impl From<String> for GlobleError {
    fn from(e: String) -> Self {
        GlobleError::ParseError(e)
    }
}
impl From<&str> for GlobleError {
    fn from(e: &str) -> Self {
        GlobleError::ParseError(format!("{:?}", e))
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Item {
    pub item_name: String,
    pub id: String,
    pub url_name: String,
    pub thumb: String,
    pub set_items: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub mod_max_rank: Option<i64>,
    pub subtypes: Option<Vec<String>>,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ItemDetails {
    pub id: String,
    pub items_in_set: Vec<ItemInfo>,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ItemInfo {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "mod_max_rank")]
    pub mod_max_rank: Option<f64>,
}
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Order {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "platinum")]
    pub platinum: i64,
    #[serde(rename = "visible")]
    pub visible: bool,

    #[serde(rename = "last_update")]
    pub last_update: String,

    #[serde(rename = "region")]
    pub region: String,

    #[serde(rename = "platform")]
    pub platform: String,

    #[serde(rename = "creation_date")]
    pub creation_date: String,

    #[serde(rename = "order_type")]
    pub order_type: String,

    #[serde(rename = "quantity")]
    pub quantity: i64,

    #[serde(rename = "item")]
    pub item: OrderItem,
}
#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct OrderItem {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "url_name")]
    pub url_name: String,
    #[serde(rename = "icon")]
    pub icon: String,

    #[serde(rename = "icon_format")]
    pub icon_format: String,

    #[serde(rename = "thumb")]
    pub thumb: String,

    #[serde(rename = "sub_icon")]
    pub sub_icon: Option<String>,

    #[serde(rename = "mod_max_rank")]
    pub mod_max_rank: Option<i64>,

    #[serde(rename = "subtypes")]
    pub subtypes: Option<Vec<String>>,

    #[serde(rename = "tags")]
    pub tags: Vec<String>,

    #[serde(rename = "ducats")]
    pub ducats: Option<i64>,

    #[serde(rename = "quantity_for_set")]
    pub quantity_for_set: Option<i64>,

    #[serde(rename = "en")]
    pub en: OrderItemTranslation,
}
#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct OrderItemTranslation {
    #[serde(rename = "item_name")]
    item_name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Ordres {
    #[serde(rename = "sell_orders")]
    pub sell_orders: Vec<Order>,
    #[serde(rename = "buy_orders")]
    pub buy_orders: Vec<Order>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Invantory {
    pub id: i64,
    pub item_id: String,
    pub item_url: String,
    pub item_name: String,
    pub rank: i64,
    pub price: f64,
    pub listed_price: Option<i64>,
    pub owned: i64,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    pub id: i64,
    pub item_name: String,
    pub item_id: String,
    pub item_url: String,
    pub item_type: String,
    pub rank: i64,
    pub price: i64,
    pub datetime: String,
    pub transaction_type: String,
    pub quantity: i64,
}

/// Generated by https://quicktype.io
extern crate serde_json;

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderByItem {
    #[serde(rename = "order_type")]
    pub order_type: String,

    #[serde(rename = "quantity")]
    pub quantity: i64,

    #[serde(rename = "platinum")]
    pub platinum: i64,

    #[serde(rename = "mod_rank")]
    pub mod_rank: Option<i64>,

    #[serde(rename = "user")]
    pub user: User,
    #[serde(rename = "platform")]
    pub platform: String,

    #[serde(rename = "creation_date")]
    pub creation_date: String,

    #[serde(rename = "last_update")]
    pub last_update: String,

    #[serde(rename = "visible")]
    pub visible: bool,

    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "region")]
    pub region: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    #[serde(rename = "reputation")]
    pub reputation: i64,

    // #[serde(rename = "locale")]
    // pub locale: String,

    // #[serde(rename = "avatar")]
    // pub avatar: String,

    // #[serde(rename = "last_seen")]
    // pub last_seen: String,
    #[serde(rename = "ingame_name")]
    pub ingame_name: String,

    #[serde(rename = "id")]
    pub id: String,
    // #[serde(rename = "region")]
    // pub region: String,
    #[serde(rename = "status")]
    pub status: String,
}

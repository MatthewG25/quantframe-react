use std::sync::{Arc, Mutex};

use polars::{
    prelude::{DataFrame, NamedFrom},
    series::Series,
};
use reqwest::{header::HeaderMap, Client, Method, Url};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

use crate::{
    auth::AuthState,
    logger,
    structs::{GlobleError, Item, ItemDetails, Order, OrderByItem, Ordres},
};

#[derive(Clone, Debug)]
pub struct WFMClientState {
    endpoint: String,
    log_file: String,
    auth: Arc<Mutex<AuthState>>,
}

impl WFMClientState {
    pub fn new(auth: Arc<Mutex<AuthState>>) -> Self {
        WFMClientState {
            endpoint: "https://api.warframe.market/v1/".to_string(),
            log_file: "wfmAPICalls.log".to_string(),
            auth,
        }
    }
    async fn send_request<T: DeserializeOwned>(
        &self,
        method: Method,
        url: &str,
        payload_key: Option<&str>,
        body: Option<Value>,
    ) -> Result<(T, HeaderMap), GlobleError> {
        let auth = self.auth.lock()?.clone();
        // Sleep for 1 seconds before sending a new request, to avoid 429 error
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        let client = Client::new();
        let new_url = format!("{}{}", self.endpoint, url);

        let request = client
            .request(method, Url::parse(&new_url).unwrap())
            .header(
                "Authorization",
                format!("JWT {}", auth.access_token.unwrap_or("".to_string())),
            )
            .header("Language", "en");

        let request = match body.clone() {
            Some(content) => request.json(&content),
            None => request,
        };
        // let response: Value = request.send().await?.json().await;
        let response = request.send().await;

        if let Err(e) = response {
            return Err(GlobleError::ReqwestError(e));
        }
        let response_data = response.unwrap();
        let status = response_data.status();

        if status == 429 {
            // Sleep for 3 second
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            return Err(GlobleError::TooManyRequests(
                "Too Many Requests".to_string(),
            ));
        }
        if status != 200 {
            let rep = response_data.text().await.unwrap();
            return Err(GlobleError::OtherError(format!(
                "URL: {}, Body: {}, Status: {}, Response: {}",
                new_url,
                body.unwrap_or(json!({})),
                status,
                rep
            )));
        }

        let headers = response_data.headers().clone();
        let response = response_data.json::<Value>().await.unwrap();

        let mut data = response["payload"].clone();
        if let Some(payload_key) = payload_key {
            data = response["payload"][payload_key].clone();
            // let payload: T =
            //     serde_json::from_value(response["payload"][payload_key].clone()).unwrap();
        }

        // Convert the response to a T object
        match serde_json::from_value(data.clone()) {
            Ok(payload) => Ok((payload, headers)),
            Err(e) => Err(GlobleError::SerdeError(
                format!("{}", data.to_string()),
                e.line(),
                e.column(),
            )),
        }
    }

    async fn get<T: DeserializeOwned>(
        &self,
        url: &str,
        payload_key: Option<&str>,
    ) -> Result<(T, HeaderMap), GlobleError> {
        let payload: (T, HeaderMap) = self
            .send_request(Method::GET, url, payload_key, None)
            .await?;
        Ok(payload)
    }

    async fn post<T: DeserializeOwned>(
        &self,
        url: &str,
        payload_key: Option<&str>,
        body: Value,
    ) -> Result<(T, HeaderMap), GlobleError> {
        let payload: (T, HeaderMap) = self
            .send_request(Method::POST, url, payload_key, Some(body))
            .await?;
        Ok(payload)
    }

    async fn delete<T: DeserializeOwned>(
        &self,
        url: &str,
        payload_key: Option<&str>,
    ) -> Result<(T, HeaderMap), GlobleError> {
        let payload: (T, HeaderMap) = self
            .send_request(Method::DELETE, url, payload_key, None)
            .await?;
        Ok(payload)
    }

    async fn put<T: DeserializeOwned>(
        &self,
        url: &str,
        payload_key: Option<&str>,
        body: Option<Value>,
    ) -> Result<(T, HeaderMap), GlobleError> {
        let payload: (T, HeaderMap) = self
            .send_request(Method::PUT, url, payload_key, body)
            .await?;
        Ok(payload)
    }

    pub async fn login(&self, email: String, password: String) -> Result<AuthState, GlobleError> {
        let body = json!({
            "email": email,
            "password": password
        });
        let (mut user, headers): (AuthState, HeaderMap) =
            self.post("/auth/signin", Some("user"), body).await?;

        // Get the "set-cookie" header
        let cookies = headers.get("set-cookie");
        // Check if the header is present
        if let Some(cookie_value) = cookies {
            // Convert HeaderValue to String
            let cookie_str = cookie_value.to_str().unwrap_or_default();

            // The slicing and splitting logic
            let access_token: Option<String> =
                Some(cookie_str[4..].split(';').next().unwrap_or("").to_string());
            user.access_token = access_token;
            user.avatar = format!("https://warframe.market/static/assets/{}", user.avatar);
        } else {
            user.clone().access_token = None;
        }
        Ok(user)
    }

    pub async fn validate(&self) -> Result<bool, GlobleError> {
        match self
            .post_ordre(
                "Lex Prime Set",
                "56783f24cbfa8f0432dd89a2",
                "buy",
                1,
                1,
                false,
                None,
            )
            .await
        {
            Ok(order) => {
                self.delete_order(
                    &order.id.clone(),
                    "Lex Prime Set",
                    "56783f24cbfa8f0432dd89a2",
                    "buy",
                )
                .await?;
                Ok(true)
            }
            Err(_e) => Ok(false),
        }
    }
    pub async fn get_tradable_items(&self) -> Result<Vec<Item>, GlobleError> {
        let (payload, _headers) = self.get("items", Some("items")).await?;
        Ok(payload)
    }
    pub async fn get_item(&self, item: String) -> Result<ItemDetails, GlobleError> {
        let url = format!("items/{}", item);
        match self.get(&url, Some("item")).await {
            Ok((item, _headers)) => {
                logger::info(
                    "WarframeMarket:GetItem",
                    format!("For Item: {:?}", item).as_str(),
                    true,
                    Some(self.log_file.as_str()),
                );
                Ok(item)
            }
            Err(e) => {
                logger::error(
                    "WarframeMarket:GetItem",
                    format!("Item: {}, Error: {:?}", item, e).as_str(),
                    true,
                    Some(self.log_file.as_str()),
                );
                Err(e)
            }
        }
    }
    // Get orders from warframe market
    pub async fn get_user_ordres(&self) -> Result<Ordres, GlobleError> {
        let auth = self.auth.lock()?.clone();
        let url = format!("profile/{}/orders", auth.ingame_name.clone());
        match self.get(&url, None).await {
            Ok((orders, _headers)) => {
                logger::info(
                    "WarframeMarket:GetUserOrdres",
                    format!("From User: {}", auth.ingame_name.clone()).as_str(),
                    true,
                    Some(self.log_file.as_str()),
                );
                Ok(orders)
            }
            Err(e) => {
                logger::error(
                    "WarframeMarket:GetUserOrdres",
                    format!("User: {}, Error: {:?}", auth.ingame_name, e).as_str(),
                    true,
                    None,
                );
                Err(e)
            }
        }
    }
    pub async fn get_ordres_data_frames(&self) -> Result<(DataFrame, DataFrame), GlobleError> {
        let current_orders = self.get_user_ordres().await?;
        let buy_orders = current_orders.buy_orders.clone();
        let my_buy_orders_df = DataFrame::new_no_checks(vec![
            // Assuming Order has fields field1, field2, ...
            Series::new(
                "id",
                buy_orders
                    .iter()
                    .map(|order| order.id.clone())
                    .collect::<Vec<_>>(),
            ),
            Series::new(
                "visible",
                buy_orders
                    .iter()
                    .map(|order| order.visible.clone())
                    .collect::<Vec<_>>(),
            ),
            Series::new(
                "url_name",
                buy_orders
                    .iter()
                    .map(|order| order.item.url_name.clone())
                    .collect::<Vec<_>>(),
            ),
            Series::new(
                "platinum",
                buy_orders
                    .iter()
                    .map(|order| order.platinum.clone())
                    .collect::<Vec<_>>(),
            ),
            Series::new(
                "platform",
                buy_orders
                    .iter()
                    .map(|order| order.platform.clone())
                    .collect::<Vec<_>>(),
            ),
            Series::new(
                "quantity",
                buy_orders
                    .iter()
                    .map(|order| order.quantity.clone())
                    .collect::<Vec<_>>(),
            ),
            Series::new(
                "last_update",
                buy_orders
                    .iter()
                    .map(|order| order.last_update.clone())
                    .collect::<Vec<_>>(),
            ),
            Series::new(
                "creation_date",
                buy_orders
                    .iter()
                    .map(|order| order.creation_date.clone())
                    .collect::<Vec<_>>(),
            ),
        ]);
        let sell_orders = current_orders.sell_orders.clone();
        let my_sell_orders_df = DataFrame::new_no_checks(vec![
            Series::new(
                "id",
                sell_orders
                    .iter()
                    .map(|order| order.id.clone())
                    .collect::<Vec<_>>(),
            ),
            Series::new(
                "visible",
                sell_orders
                    .iter()
                    .map(|order| order.visible.clone())
                    .collect::<Vec<_>>(),
            ),
            Series::new(
                "url_name",
                sell_orders
                    .iter()
                    .map(|order| order.item.url_name.clone())
                    .collect::<Vec<_>>(),
            ),
            Series::new(
                "platinum",
                sell_orders
                    .iter()
                    .map(|order| order.platinum.clone())
                    .collect::<Vec<_>>(),
            ),
            Series::new(
                "platform",
                sell_orders
                    .iter()
                    .map(|order| order.platform.clone())
                    .collect::<Vec<_>>(),
            ),
            Series::new(
                "quantity",
                sell_orders
                    .iter()
                    .map(|order| order.quantity.clone())
                    .collect::<Vec<_>>(),
            ),
            Series::new(
                "last_update",
                sell_orders
                    .iter()
                    .map(|order| order.last_update.clone())
                    .collect::<Vec<_>>(),
            ),
            Series::new(
                "creation_date",
                sell_orders
                    .iter()
                    .map(|order| order.creation_date.clone())
                    .collect::<Vec<_>>(),
            ),
        ]);
        Ok((my_buy_orders_df, my_sell_orders_df))
    }
    pub async fn post_ordre(
        &self,
        item_name: &str,
        item_id: &str,
        order_type: &str,
        platinum: i64,
        quantity: i64,
        visible: bool,
        rank: Option<f64>,
    ) -> Result<Order, GlobleError> {
        // Construct any JSON body
        let mut body = json!({
            "item": item_id,
            "order_type": order_type,
            "platinum": platinum,
            "quantity": quantity,
            "visible": visible
        });
        // Add rank to body if it exists
        if let Some(rank) = rank {
            body["rank"] = json!(rank);
        }
        match self.post("profile/orders", Some("order"), body).await {
            Ok((order, _headers)) => {
                logger::info("WarframeMarket:PostOrder", format!("Created Order: {}, Item Name: {}, Item Id: {},  Platinum: {}, Quantity: {}, Visible: {}", order_type, item_name, item_id ,platinum ,quantity ,visible).as_str(), true, Some(self.log_file.as_str()));
                Ok(order)
            }
            Err(e) => {
                logger::error(
                    "WarframeMarket:PostOrder",
                    format!("{:?}", e).as_str(),
                    true,
                    Some(&self.log_file),
                );
                Err(e)
            }
        }
    }
    pub async fn delete_order(
        &self,
        order_id: &str,
        item_name: &str,
        item_id: &str,
        order_type: &str,
    ) -> Result<String, GlobleError> {
        let url = format!("profile/orders/{}", order_id);
        match self.delete(&url, Some("order_id")).await {
            Ok((order_id, _headers)) => {
                logger::info(
                    "WarframeMarket:DeleteOrder",
                    format!(
                        "Deleted order: {}, Item Name: {}, Item Id: {}, Type: {}",
                        order_id, item_name, item_id, order_type
                    )
                    .as_str(),
                    true,
                    Some(self.log_file.as_str()),
                );
                Ok(order_id)
            }
            Err(e) => {
                logger::error(
                    "WarframeMarket:DeleteOrder",
                    format!("{:?}", e).as_str(),
                    true,
                    None,
                );
                Err(e)
            }
        }
    }
    pub fn convet_order_to_datafream(&self, order: Order) -> Result<DataFrame, GlobleError> {
        let orders_df = DataFrame::new_no_checks(vec![
            Series::new("id", vec![order.id.clone()]),
            Series::new("visible", vec![order.visible.clone()]),
            Series::new("url_name", vec![order.item.url_name.clone()]),
            Series::new("platinum", vec![order.platinum.clone()]),
            Series::new("platform", vec![order.platform.clone()]),
            Series::new("quantity", vec![order.quantity.clone()]),
            Series::new("last_update", vec![order.last_update.clone()]),
            Series::new("creation_date", vec![order.creation_date.clone()]),
        ]);
        Ok(orders_df)
    }
    pub async fn update_order_listing(
        &self,
        order_id: &str,
        platinum: i64,
        quantity: i64,
        visible: bool,
        item_name: &str,
        item_id: &str,
        order_type: &str,
    ) -> Result<Order, GlobleError> {
        // Construct any JSON body
        let body = json!({
            "platinum": platinum,
            "quantity": quantity,
            "visible": visible
        });
        let url = format!("profile/orders/{}", order_id);
        match self.put(&url, Some("order"), Some(body)).await {
            Ok((order, _headers)) => {
                logger::info("WarframeMarket:UpdateOrderListing", format!("Updated Order Id: {}, Item Name: {}, Item Id: {}, Platinum: {}, Quantity: {}, Visible: {}, Type: {}", order_id, item_name, item_id,platinum ,quantity ,visible, order_type).as_str(), true, Some(&self.log_file));
                Ok(order)
            }
            Err(e) => {
                logger::error(
                    "WarframeMarket:UpdateOrderListing",
                    format!("{:?}", e).as_str(),
                    true,
                    Some(self.log_file.as_str()),
                );
                Err(e)
            }
        }
    }

    pub async fn get_ordres_by_item(&self, item: &str) -> Result<DataFrame, GlobleError> {
        let url = format!("items/{}/orders", item);

        let orders: Vec<OrderByItem> = match self.get(&url, Some("orders")).await {
            Ok((orders, _headers)) => orders,
            Err(e) => {
                logger::error(
                    "WarframeMarket:GetOrdresByItem",
                    format!("Item: {}, Error: {:?}", item, e).as_str(),
                    true,
                    Some(self.log_file.as_str()),
                );
                vec![]
            }
        };

        if orders.len() == 0 {
            return Ok(DataFrame::new_no_checks(vec![]));
        }

        let mod_rank = orders
            .iter()
            .max_by(|a, b| a.mod_rank.cmp(&b.mod_rank))
            .unwrap()
            .mod_rank;

        let orders: Vec<OrderByItem> = orders
            .into_iter()
            .filter(|order| order.user.status == "ingame" && order.mod_rank == mod_rank)
            .collect();

        // Check if mod_rank is null
        let orders_df = DataFrame::new_no_checks(vec![
            Series::new(
                "username",
                orders
                    .iter()
                    .map(|order| order.user.ingame_name.clone())
                    .collect::<Vec<_>>(),
            ),
            Series::new(
                "platinum",
                orders
                    .iter()
                    .map(|order| order.platinum.clone())
                    .collect::<Vec<_>>(),
            ),
            Series::new(
                "mod_rank",
                orders
                    .iter()
                    .map(|order| order.mod_rank.clone())
                    .collect::<Vec<_>>(),
            ),
            Series::new(
                "username",
                orders
                    .iter()
                    .map(|order| order.user.ingame_name.clone())
                    .collect::<Vec<_>>(),
            ),
            Series::new(
                "order_type",
                orders
                    .iter()
                    .map(|order| order.order_type.clone())
                    .collect::<Vec<_>>(),
            ),
        ]);
        Ok(orders_df)
    }

    pub async fn close_order_by_url(&self, item: &str) -> Result<String, GlobleError> {
        // Get the user orders and find the order
        let mut ordres_vec = self.get_user_ordres().await?;
        let mut ordres: Vec<Order> = ordres_vec.buy_orders;
        ordres.append(&mut ordres_vec.sell_orders);

        let order = ordres
            .iter()
            .find(|order| order.item.url_name == item)
            .clone();

        if order.is_none() {
            return Ok("No Order Found".to_string());
        }

        let url = format!("profile/orders/close/{}", order.unwrap().id);
        let result: Result<(Option<String>, HeaderMap), GlobleError> =
            self.put(&url, Some("order_id"), None).await;
        match result {
            Ok((order_data, _headers)) => {
                logger::info(
                    "WarframeMarket:CloseOrder",
                    format!("Closed Order: {}", order.unwrap().id).as_str(),
                    true,
                    Some(self.log_file.as_str()),
                );
                Ok(order_data.unwrap_or("Order Successfully Closed".to_string()))
            }
            Err(e) => {
                logger::error(
                    "WarframeMarket:CloseOrder",
                    format!("{:?}", e).as_str(),
                    true,
                    Some(self.log_file.as_str()),
                );
                Err(e)
            }
        }
    }
}

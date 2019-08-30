use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Board {
    pub mid_price: f64,
    pub bids: Vec<BoardOrder>,
    pub asks: Vec<BoardOrder>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BoardOrder {
    pub price: f64,
    pub size: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExchangeHistory {
    pub id: i64,
    pub side: String,
    pub price: f64,
    pub size: f64,
    pub exec_date: String,
    pub buy_child_order_acceptance_id: String,
    pub sell_child_order_acceptance_id: String,
}

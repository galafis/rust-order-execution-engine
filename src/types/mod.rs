use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Order side (Buy or Sell)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Side::Buy => write!(f, "BUY"),
            Side::Sell => write!(f, "SELL"),
        }
    }
}

/// Order type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    StopLimit,
}

/// Order status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

/// Financial order representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub quantity: u64,
    pub price: Option<f64>,
    pub stop_price: Option<f64>,
    pub filled_quantity: u64,
    pub status: OrderStatus,
    pub timestamp: DateTime<Utc>,
    pub client_id: String,
}

impl Order {
    pub fn new_market(symbol: String, side: Side, quantity: u64, client_id: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            symbol,
            side,
            order_type: OrderType::Market,
            quantity,
            price: None,
            stop_price: None,
            filled_quantity: 0,
            status: OrderStatus::Pending,
            timestamp: Utc::now(),
            client_id,
        }
    }

    pub fn new_limit(
        symbol: String,
        side: Side,
        quantity: u64,
        price: f64,
        client_id: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            symbol,
            side,
            order_type: OrderType::Limit,
            quantity,
            price: Some(price),
            stop_price: None,
            filled_quantity: 0,
            status: OrderStatus::Pending,
            timestamp: Utc::now(),
            client_id,
        }
    }

    pub fn remaining_quantity(&self) -> u64 {
        self.quantity.saturating_sub(self.filled_quantity)
    }

    pub fn is_fully_filled(&self) -> bool {
        self.filled_quantity >= self.quantity
    }
}

/// Trade execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: Uuid,
    pub buy_order_id: Uuid,
    pub sell_order_id: Uuid,
    pub symbol: String,
    pub quantity: u64,
    pub price: f64,
    pub timestamp: DateTime<Utc>,
}

impl Trade {
    pub fn new(
        buy_order_id: Uuid,
        sell_order_id: Uuid,
        symbol: String,
        quantity: u64,
        price: f64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            buy_order_id,
            sell_order_id,
            symbol,
            quantity,
            price,
            timestamp: Utc::now(),
        }
    }
}

/// Execution metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub total_orders: u64,
    pub filled_orders: u64,
    pub cancelled_orders: u64,
    pub rejected_orders: u64,
    pub total_trades: u64,
    pub total_volume: f64,
    pub avg_latency_micros: u64,
    pub p50_latency_micros: u64,
    pub p95_latency_micros: u64,
    pub p99_latency_micros: u64,
}

impl ExecutionMetrics {
    pub fn fill_rate(&self) -> f64 {
        if self.total_orders == 0 {
            0.0
        } else {
            (self.filled_orders as f64 / self.total_orders as f64) * 100.0
        }
    }
}

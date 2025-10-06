use crate::types::{Order, OrderStatus, Side, Trade};
use std::collections::{BTreeMap, VecDeque};
use uuid::Uuid;

/// Order book for a single symbol
#[derive(Debug)]
pub struct OrderBook {
    symbol: String,
    bids: BTreeMap<u64, VecDeque<Order>>, // Price level -> Orders (sorted by price descending)
    asks: BTreeMap<u64, VecDeque<Order>>, // Price level -> Orders (sorted by price ascending)
}

impl OrderBook {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    /// Add order to the book
    pub fn add_order(&mut self, order: Order) {
        let price_level = (order.price.unwrap_or(0.0) * 100.0) as u64; // Convert to integer for BTreeMap
        
        match order.side {
            Side::Buy => {
                self.bids
                    .entry(price_level)
                    .or_insert_with(VecDeque::new)
                    .push_back(order);
            }
            Side::Sell => {
                self.asks
                    .entry(price_level)
                    .or_insert_with(VecDeque::new)
                    .push_back(order);
            }
        }
    }

    /// Match orders and generate trades
    pub fn match_orders(&mut self) -> Vec<Trade> {
        let mut trades = Vec::new();

        loop {
            // Get best bid and ask
            let best_bid_price = self.bids.keys().next_back().copied();
            let best_ask_price = self.asks.keys().next().copied();

            match (best_bid_price, best_ask_price) {
                (Some(bid_price), Some(ask_price)) if bid_price >= ask_price => {
                    // Match possible
                    let mut bid_orders = self.bids.remove(&bid_price).unwrap();
                    let mut ask_orders = self.asks.remove(&ask_price).unwrap();

                    while let (Some(bid), Some(ask)) =
                        (bid_orders.front_mut(), ask_orders.front_mut())
                    {
                        let trade_quantity = bid.remaining_quantity().min(ask.remaining_quantity());
                        let trade_price = (ask_price as f64) / 100.0;

                        // Create trade
                        let trade = Trade::new(
                            bid.id,
                            ask.id,
                            self.symbol.clone(),
                            trade_quantity,
                            trade_price,
                        );

                        // Update orders
                        bid.filled_quantity += trade_quantity;
                        ask.filled_quantity += trade_quantity;

                        if bid.is_fully_filled() {
                            bid.status = OrderStatus::Filled;
                            bid_orders.pop_front();
                        } else {
                            bid.status = OrderStatus::PartiallyFilled;
                        }

                        if ask.is_fully_filled() {
                            ask.status = OrderStatus::Filled;
                            ask_orders.pop_front();
                        } else {
                            ask.status = OrderStatus::PartiallyFilled;
                        }

                        trades.push(trade);

                        if bid_orders.is_empty() || ask_orders.is_empty() {
                            break;
                        }
                    }

                    // Put back remaining orders
                    if !bid_orders.is_empty() {
                        self.bids.insert(bid_price, bid_orders);
                    }
                    if !ask_orders.is_empty() {
                        self.asks.insert(ask_price, ask_orders);
                    }
                }
                _ => break, // No more matches possible
            }
        }

        trades
    }

    /// Cancel order by ID
    pub fn cancel_order(&mut self, order_id: Uuid) -> Option<Order> {
        // Search in bids
        for orders in self.bids.values_mut() {
            if let Some(pos) = orders.iter().position(|o| o.id == order_id) {
                let mut order = orders.remove(pos).unwrap();
                order.status = OrderStatus::Cancelled;
                return Some(order);
            }
        }

        // Search in asks
        for orders in self.asks.values_mut() {
            if let Some(pos) = orders.iter().position(|o| o.id == order_id) {
                let mut order = orders.remove(pos).unwrap();
                order.status = OrderStatus::Cancelled;
                return Some(order);
            }
        }

        None
    }

    /// Get current best bid price
    pub fn best_bid(&self) -> Option<f64> {
        self.bids.keys().next_back().map(|&p| (p as f64) / 100.0)
    }

    /// Get current best ask price
    pub fn best_ask(&self) -> Option<f64> {
        self.asks.keys().next().map(|&p| (p as f64) / 100.0)
    }

    /// Get mid price
    pub fn mid_price(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some((bid + ask) / 2.0),
            _ => None,
        }
    }

    /// Get total depth (number of orders)
    pub fn depth(&self) -> usize {
        let bid_depth: usize = self.bids.values().map(|v| v.len()).sum();
        let ask_depth: usize = self.asks.values().map(|v| v.len()).sum();
        bid_depth + ask_depth
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Side;

    #[test]
    fn test_order_book_creation() {
        let book = OrderBook::new("BTCUSD".to_string());
        assert_eq!(book.depth(), 0);
    }

    #[test]
    fn test_add_orders() {
        let mut book = OrderBook::new("BTCUSD".to_string());
        
        let buy_order = Order::new_limit("BTCUSD".to_string(), Side::Buy, 10, 50000.0, "client1".to_string());
        let sell_order = Order::new_limit("BTCUSD".to_string(), Side::Sell, 5, 50100.0, "client2".to_string());
        
        book.add_order(buy_order);
        book.add_order(sell_order);
        
        assert_eq!(book.depth(), 2);
        assert_eq!(book.best_bid(), Some(50000.0));
        assert_eq!(book.best_ask(), Some(50100.0));
    }

    #[test]
    fn test_order_matching() {
        let mut book = OrderBook::new("BTCUSD".to_string());
        
        let buy_order = Order::new_limit("BTCUSD".to_string(), Side::Buy, 10, 50000.0, "client1".to_string());
        let sell_order = Order::new_limit("BTCUSD".to_string(), Side::Sell, 5, 49900.0, "client2".to_string());
        
        book.add_order(buy_order);
        book.add_order(sell_order);
        
        let trades = book.match_orders();
        
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].quantity, 5);
        assert_eq!(trades[0].price, 49900.0);
    }
}

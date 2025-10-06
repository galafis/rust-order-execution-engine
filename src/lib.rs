//! # Rust Order Execution Engine
//!
//! A high-performance order execution engine for financial markets, built with Rust.
//!
//! ## Features
//!
//! - **Low Latency**: Optimized for microsecond-level order processing
//! - **Async/Await**: Built on Tokio for efficient concurrent operations
//! - **Thread-Safe**: Lock-free data structures where possible
//! - **Order Matching**: FIFO price-time priority matching algorithm
//! - **Metrics**: Comprehensive execution metrics including latency percentiles
//!
//! ## Example
//!
//! ```rust
//! use rust_order_execution_engine::{ExecutionEngine, Order, Side};
//! use crossbeam::channel::unbounded;
//!
//! #[tokio::main]
//! async fn main() {
//!     let (trade_sender, trade_receiver) = unbounded();
//!     let engine = ExecutionEngine::new(trade_sender);
//!     
//!     engine.start().await;
//!     
//!     let order = Order::new_limit(
//!         "BTCUSD".to_string(),
//!         Side::Buy,
//!         10,
//!         50000.0,
//!         "client1".to_string()
//!     );
//!     
//!     engine.submit_order(order).await.unwrap();
//!     
//!     // Process trades
//!     while let Ok(trade) = trade_receiver.recv() {
//!         println!("Trade executed: {:?}", trade);
//!     }
//! }
//! ```

pub mod engine;
pub mod matching;
pub mod types;

pub use engine::{ExecutionEngine, EngineError};
pub use matching::OrderBook;
pub use types::{ExecutionMetrics, Order, OrderStatus, OrderType, Side, Trade};

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam::channel::unbounded;

    #[tokio::test]
    async fn test_engine_lifecycle() {
        let (trade_sender, _trade_receiver) = unbounded();
        let engine = ExecutionEngine::new(trade_sender);
        
        engine.start().await;
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        let metrics = engine.get_metrics();
        assert_eq!(metrics.total_orders, 0);
        
        engine.stop().await;
    }

    #[tokio::test]
    async fn test_submit_order() {
        let (trade_sender, _trade_receiver) = unbounded();
        let engine = ExecutionEngine::new(trade_sender);
        
        engine.start().await;
        
        let order = Order::new_limit(
            "BTCUSD".to_string(),
            Side::Buy,
            10,
            50000.0,
            "client1".to_string()
        );
        
        let result = engine.submit_order(order).await;
        assert!(result.is_ok());
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        let metrics = engine.get_metrics();
        assert_eq!(metrics.total_orders, 1);
        
        engine.stop().await;
    }

    #[tokio::test]
    async fn test_order_matching_integration() {
        let (trade_sender, trade_receiver) = unbounded();
        let engine = ExecutionEngine::new(trade_sender);
        
        engine.start().await;
        
        // Submit buy order
        let buy_order = Order::new_limit(
            "BTCUSD".to_string(),
            Side::Buy,
            10,
            50000.0,
            "client1".to_string()
        );
        engine.submit_order(buy_order).await.unwrap();
        
        // Submit sell order that matches
        let sell_order = Order::new_limit(
            "BTCUSD".to_string(),
            Side::Sell,
            5,
            49900.0,
            "client2".to_string()
        );
        engine.submit_order(sell_order).await.unwrap();
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Check for trade
        let trade = trade_receiver.try_recv();
        assert!(trade.is_ok());
        
        let metrics = engine.get_metrics();
        assert_eq!(metrics.total_orders, 2);
        assert_eq!(metrics.total_trades, 1);
        
        engine.stop().await;
    }
}

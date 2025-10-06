use crate::matching::OrderBook;
use crate::types::{ExecutionMetrics, Order, OrderStatus, OrderType, Trade};
use crossbeam::channel::{bounded, Receiver, Sender};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::task;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Invalid order: {0}")]
    InvalidOrder(String),
    
    #[error("Order not found: {0}")]
    OrderNotFound(Uuid),
    
    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),
    
    #[error("Engine is stopped")]
    EngineStopped,
}

pub type Result<T> = std::result::Result<T, EngineError>;

/// Main execution engine
pub struct ExecutionEngine {
    order_books: Arc<Mutex<HashMap<String, OrderBook>>>,
    order_sender: Sender<EngineCommand>,
    order_receiver: Arc<Mutex<Receiver<EngineCommand>>>,
    trade_sender: Sender<Trade>,
    metrics: Arc<Mutex<ExecutionMetrics>>,
    latency_samples: Arc<Mutex<Vec<u64>>>,
    running: Arc<Mutex<bool>>,
}

enum EngineCommand {
    NewOrder(Order),
    CancelOrder(Uuid, String),
    Shutdown,
}

impl ExecutionEngine {
    pub fn new(trade_sender: Sender<Trade>) -> Self {
        let (order_sender, order_receiver) = bounded(10000);
        
        Self {
            order_books: Arc::new(Mutex::new(HashMap::new())),
            order_sender,
            order_receiver: Arc::new(Mutex::new(order_receiver)),
            trade_sender,
            metrics: Arc::new(Mutex::new(ExecutionMetrics::default())),
            latency_samples: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Start the execution engine
    pub async fn start(&self) {
        let mut running = self.running.lock().unwrap();
        if *running {
            warn!("Engine already running");
            return;
        }
        *running = true;
        drop(running);

        info!("Starting execution engine");

        let order_receiver = Arc::clone(&self.order_receiver);
        let order_books = Arc::clone(&self.order_books);
        let trade_sender = self.trade_sender.clone();
        let metrics = Arc::clone(&self.metrics);
        let latency_samples = Arc::clone(&self.latency_samples);
        let running = Arc::clone(&self.running);

        task::spawn(async move {
            loop {
                if !*running.lock().unwrap() {
                    info!("Engine stopping");
                    break;
                }

                let receiver = order_receiver.lock().unwrap();
                let command = receiver.recv_timeout(Duration::from_millis(100));
                drop(receiver);

                match command {
                    Ok(EngineCommand::NewOrder(order)) => {
                        let start = Instant::now();
                        Self::process_order(
                            order,
                            &order_books,
                            &trade_sender,
                            &metrics,
                            &latency_samples,
                        );
                        let latency = start.elapsed().as_micros() as u64;
                        latency_samples.lock().unwrap().push(latency);
                    }
                    Ok(EngineCommand::CancelOrder(order_id, symbol)) => {
                        Self::process_cancel(order_id, symbol, &order_books, &metrics);
                    }
                    Ok(EngineCommand::Shutdown) => {
                        info!("Received shutdown command");
                        break;
                    }
                    Err(_) => {
                        // Timeout, continue
                        continue;
                    }
                }
            }
        });
    }

    fn process_order(
        mut order: Order,
        order_books: &Arc<Mutex<HashMap<String, OrderBook>>>,
        trade_sender: &Sender<Trade>,
        metrics: &Arc<Mutex<ExecutionMetrics>>,
        _latency_samples: &Arc<Mutex<Vec<u64>>>,
    ) {
        debug!("Processing order: {:?}", order.id);

        // Validate order
        if order.quantity == 0 {
            error!("Invalid order quantity: 0");
            order.status = OrderStatus::Rejected;
            metrics.lock().unwrap().rejected_orders += 1;
            return;
        }

        if order.order_type == OrderType::Limit && order.price.is_none() {
            error!("Limit order without price");
            order.status = OrderStatus::Rejected;
            metrics.lock().unwrap().rejected_orders += 1;
            return;
        }

        let mut books = order_books.lock().unwrap();
        let book = books
            .entry(order.symbol.clone())
            .or_insert_with(|| OrderBook::new(order.symbol.clone()));

        // Add order to book
        book.add_order(order.clone());

        // Try to match orders
        let trades = book.match_orders();

        // Update metrics
        let mut metrics_guard = metrics.lock().unwrap();
        metrics_guard.total_orders += 1;

        if !trades.is_empty() {
            metrics_guard.total_trades += trades.len() as u64;
            for trade in &trades {
                metrics_guard.total_volume += trade.quantity as f64 * trade.price;
            }
            metrics_guard.filled_orders += 1;
        }

        drop(metrics_guard);
        drop(books);

        // Send trades
        for trade in trades {
            if let Err(e) = trade_sender.try_send(trade) {
                error!("Failed to send trade: {}", e);
            }
        }
    }

    fn process_cancel(
        order_id: Uuid,
        symbol: String,
        order_books: &Arc<Mutex<HashMap<String, OrderBook>>>,
        metrics: &Arc<Mutex<ExecutionMetrics>>,
    ) {
        debug!("Cancelling order: {:?}", order_id);

        let mut books = order_books.lock().unwrap();
        if let Some(book) = books.get_mut(&symbol) {
            if let Some(_cancelled_order) = book.cancel_order(order_id) {
                metrics.lock().unwrap().cancelled_orders += 1;
                info!("Order cancelled: {:?}", order_id);
            } else {
                warn!("Order not found for cancellation: {:?}", order_id);
            }
        } else {
            warn!("Symbol not found: {}", symbol);
        }
    }

    /// Submit new order
    pub async fn submit_order(&self, order: Order) -> Result<()> {
        if !*self.running.lock().unwrap() {
            return Err(EngineError::EngineStopped);
        }

        self.order_sender
            .send(EngineCommand::NewOrder(order))
            .map_err(|_| EngineError::EngineStopped)?;

        Ok(())
    }

    /// Cancel order
    pub async fn cancel_order(&self, order_id: Uuid, symbol: String) -> Result<()> {
        if !*self.running.lock().unwrap() {
            return Err(EngineError::EngineStopped);
        }

        self.order_sender
            .send(EngineCommand::CancelOrder(order_id, symbol))
            .map_err(|_| EngineError::EngineStopped)?;

        Ok(())
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> ExecutionMetrics {
        let mut metrics = self.metrics.lock().unwrap().clone();
        
        // Calculate latency percentiles
        let mut samples = self.latency_samples.lock().unwrap();
        if !samples.is_empty() {
            samples.sort_unstable();
            let len = samples.len();
            
            metrics.avg_latency_micros = samples.iter().sum::<u64>() / len as u64;
            metrics.p50_latency_micros = samples[len / 2];
            metrics.p95_latency_micros = samples[(len * 95) / 100];
            metrics.p99_latency_micros = samples[(len * 99) / 100];
        }
        
        metrics
    }

    /// Stop the engine
    pub async fn stop(&self) {
        info!("Stopping execution engine");
        let mut running = self.running.lock().unwrap();
        *running = false;
        drop(running);

        let _ = self.order_sender.send(EngineCommand::Shutdown);
    }

    /// Get order book for symbol
    pub fn get_order_book(&self, symbol: &str) -> Option<(Option<f64>, Option<f64>, usize)> {
        let books = self.order_books.lock().unwrap();
        books.get(symbol).map(|book| {
            (book.best_bid(), book.best_ask(), book.depth())
        })
    }
}

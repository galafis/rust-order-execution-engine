use crossbeam::channel::unbounded;
use rust_order_execution_engine::{ExecutionEngine, Order, Side};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting Order Execution Engine Example");

    // Create trade channel
    let (trade_sender, trade_receiver) = unbounded();

    // Create and start engine
    let engine = ExecutionEngine::new(trade_sender);
    engine.start().await;

    info!("Engine started");

    // Submit buy orders
    for i in 0..5 {
        let order = Order::new_limit(
            "BTCUSD".to_string(),
            Side::Buy,
            10 + i,
            50000.0 - (i as f64 * 10.0),
            format!("buyer_{}", i),
        );
        engine.submit_order(order).await.unwrap();
        info!("Submitted buy order {}", i);
    }

    // Submit sell orders
    for i in 0..5 {
        let order = Order::new_limit(
            "BTCUSD".to_string(),
            Side::Sell,
            8 + i,
            49950.0 + (i as f64 * 10.0),
            format!("seller_{}", i),
        );
        engine.submit_order(order).await.unwrap();
        info!("Submitted sell order {}", i);
    }

    // Wait for processing
    sleep(Duration::from_millis(500)).await;

    // Collect trades
    let mut trade_count = 0;
    while let Ok(trade) = trade_receiver.try_recv() {
        info!(
            "Trade #{}: {} @ {} (qty: {})",
            trade_count + 1,
            trade.symbol,
            trade.price,
            trade.quantity
        );
        trade_count += 1;
    }

    // Get metrics
    let metrics = engine.get_metrics();
    info!("=== Execution Metrics ===");
    info!("Total Orders: {}", metrics.total_orders);
    info!("Filled Orders: {}", metrics.filled_orders);
    info!("Total Trades: {}", metrics.total_trades);
    info!("Total Volume: ${:.2}", metrics.total_volume);
    info!("Fill Rate: {:.2}%", metrics.fill_rate());
    info!("Avg Latency: {} μs", metrics.avg_latency_micros);
    info!("P50 Latency: {} μs", metrics.p50_latency_micros);
    info!("P95 Latency: {} μs", metrics.p95_latency_micros);
    info!("P99 Latency: {} μs", metrics.p99_latency_micros);

    // Get order book state
    if let Some((best_bid, best_ask, depth)) = engine.get_order_book("BTCUSD") {
        info!("=== Order Book ===");
        info!("Best Bid: {:?}", best_bid);
        info!("Best Ask: {:?}", best_ask);
        info!("Total Depth: {}", depth);
    }

    // Stop engine
    engine.stop().await;
    info!("Engine stopped");
}

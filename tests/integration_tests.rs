use rust_order_execution_engine::*;

#[test]
fn test_order_creation() {
    let order = Order::new(
        "order1".to_string(),
        "BTCUSD".to_string(),
        Side::Buy,
        OrderType::Limit,
        100.0,
        Some(50000.0),
    );
    
    assert_eq!(order.id, "order1");
    assert_eq!(order.symbol, "BTCUSD");
    assert_eq!(order.quantity, 100.0);
    assert_eq!(order.price, Some(50000.0));
}

#[test]
fn test_order_matching() {
    let mut order_book = OrderBook::new("BTCUSD".to_string());
    
    // Add buy order
    let buy_order = Order::new(
        "buy1".to_string(),
        "BTCUSD".to_string(),
        Side::Buy,
        OrderType::Limit,
        100.0,
        Some(50000.0),
    );
    order_book.add_order(buy_order);
    
    // Add sell order that matches
    let sell_order = Order::new(
        "sell1".to_string(),
        "BTCUSD".to_string(),
        Side::Sell,
        OrderType::Limit,
        100.0,
        Some(50000.0),
    );
    
    let trades = order_book.match_order(sell_order);
    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].quantity, 100.0);
    assert_eq!(trades[0].price, 50000.0);
}

#[test]
fn test_partial_fill() {
    let mut order_book = OrderBook::new("BTCUSD".to_string());
    
    // Add buy order for 100
    let buy_order = Order::new(
        "buy1".to_string(),
        "BTCUSD".to_string(),
        Side::Buy,
        OrderType::Limit,
        100.0,
        Some(50000.0),
    );
    order_book.add_order(buy_order);
    
    // Add sell order for 50 (partial fill)
    let sell_order = Order::new(
        "sell1".to_string(),
        "BTCUSD".to_string(),
        Side::Sell,
        OrderType::Limit,
        50.0,
        Some(50000.0),
    );
    
    let trades = order_book.match_order(sell_order);
    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].quantity, 50.0);
    
    // Check remaining quantity in order book
    let depth = order_book.get_depth(5);
    assert_eq!(depth.bids.len(), 1);
    assert_eq!(depth.bids[0].1, 50.0); // Remaining 50
}

#[test]
fn test_market_order() {
    let mut order_book = OrderBook::new("BTCUSD".to_string());
    
    // Add limit order
    let limit_order = Order::new(
        "limit1".to_string(),
        "BTCUSD".to_string(),
        Side::Sell,
        OrderType::Limit,
        100.0,
        Some(50000.0),
    );
    order_book.add_order(limit_order);
    
    // Add market order
    let market_order = Order::new(
        "market1".to_string(),
        "BTCUSD".to_string(),
        Side::Buy,
        OrderType::Market,
        100.0,
        None,
    );
    
    let trades = order_book.match_order(market_order);
    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].quantity, 100.0);
}

#[test]
fn test_order_book_depth() {
    let mut order_book = OrderBook::new("BTCUSD".to_string());
    
    // Add multiple orders
    for i in 0..5 {
        let buy_order = Order::new(
            format!("buy{}", i),
            "BTCUSD".to_string(),
            Side::Buy,
            OrderType::Limit,
            100.0,
            Some(50000.0 - (i as f64 * 10.0)),
        );
        order_book.add_order(buy_order);
        
        let sell_order = Order::new(
            format!("sell{}", i),
            "BTCUSD".to_string(),
            Side::Sell,
            OrderType::Limit,
            100.0,
            Some(50100.0 + (i as f64 * 10.0)),
        );
        order_book.add_order(sell_order);
    }
    
    let depth = order_book.get_depth(3);
    assert_eq!(depth.bids.len(), 3);
    assert_eq!(depth.asks.len(), 3);
    
    // Check price ordering
    assert!(depth.bids[0].0 > depth.bids[1].0);
    assert!(depth.asks[0].0 < depth.asks[1].0);
}

#[test]
fn test_cancel_order() {
    let mut order_book = OrderBook::new("BTCUSD".to_string());
    
    let order = Order::new(
        "order1".to_string(),
        "BTCUSD".to_string(),
        Side::Buy,
        OrderType::Limit,
        100.0,
        Some(50000.0),
    );
    order_book.add_order(order);
    
    let depth_before = order_book.get_depth(5);
    assert_eq!(depth_before.bids.len(), 1);
    
    order_book.cancel_order("order1");
    
    let depth_after = order_book.get_depth(5);
    assert_eq!(depth_after.bids.len(), 0);
}

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rust_order_execution_engine::{Order, OrderBook, Side};

fn benchmark_order_book_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("order_book");

    // Benchmark adding orders
    group.bench_function("add_order", |b| {
        b.iter(|| {
            let mut book = OrderBook::new("BTCUSD".to_string());
            for i in 0..100 {
                let order = Order::new_limit(
                    "BTCUSD".to_string(),
                    if i % 2 == 0 { Side::Buy } else { Side::Sell },
                    10,
                    50000.0 + (i as f64),
                    format!("client_{}", i),
                );
                book.add_order(black_box(order));
            }
        });
    });

    // Benchmark order matching
    group.bench_function("match_orders", |b| {
        b.iter(|| {
            let mut book = OrderBook::new("BTCUSD".to_string());
            
            // Add buy orders
            for i in 0..50 {
                let order = Order::new_limit(
                    "BTCUSD".to_string(),
                    Side::Buy,
                    10,
                    50000.0 - (i as f64 * 10.0),
                    format!("buyer_{}", i),
                );
                book.add_order(order);
            }
            
            // Add sell orders
            for i in 0..50 {
                let order = Order::new_limit(
                    "BTCUSD".to_string(),
                    Side::Sell,
                    10,
                    49500.0 + (i as f64 * 10.0),
                    format!("seller_{}", i),
                );
                book.add_order(order);
            }
            
            black_box(book.match_orders());
        });
    });

    // Benchmark with different order counts
    for order_count in [100, 500, 1000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::new("match_orders_scale", order_count),
            order_count,
            |b, &count| {
                b.iter(|| {
                    let mut book = OrderBook::new("BTCUSD".to_string());
                    
                    for i in 0..count / 2 {
                        let buy_order = Order::new_limit(
                            "BTCUSD".to_string(),
                            Side::Buy,
                            10,
                            50000.0 - (i as f64),
                            format!("buyer_{}", i),
                        );
                        book.add_order(buy_order);
                        
                        let sell_order = Order::new_limit(
                            "BTCUSD".to_string(),
                            Side::Sell,
                            10,
                            49500.0 + (i as f64),
                            format!("seller_{}", i),
                        );
                        book.add_order(sell_order);
                    }
                    
                    black_box(book.match_orders());
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_order_book_operations);
criterion_main!(benches);

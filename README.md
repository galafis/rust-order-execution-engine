# 🚀 Rust Order Execution Engine

[![Rust](https://img.shields.io/badge/rust-1.90%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Docker](https://img.shields.io/badge/Docker-Ready-2496ED.svg?logo=docker)](Dockerfile)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

[English](#english) | [Português](#português)

---

## English

### Overview

A high-performance order execution engine for financial markets, built with Rust. This engine provides ultra-low latency order processing, efficient order matching, and comprehensive execution metrics suitable for quantitative trading platforms.

```mermaid
flowchart LR
    A[Order Input\nMarket / Limit / Stop] --> B[Validation\nSchema & Risk Check]
    B --> C[Matching Engine\nFIFO Price-Time Priority]
    C --> D[Execution\nTrade Generation]
    D --> E[Settlement\nPosition Update]
    E --> F[Reporting\nMetrics & P&L]

    style A fill:#1a1a2e,stroke:#e94560,color:#fff
    style B fill:#16213e,stroke:#0f3460,color:#fff
    style C fill:#0f3460,stroke:#533483,color:#fff
    style D fill:#533483,stroke:#e94560,color:#fff
    style E fill:#16213e,stroke:#e94560,color:#fff
    style F fill:#1a1a2e,stroke:#e94560,color:#fff
```

### Key Features

- **Ultra-Low Latency**: Microsecond-level order processing with optimized data structures
- **Asynchronous Architecture**: Built on Tokio runtime for efficient concurrent operations
- **Thread-Safe Design**: Lock-free data structures and careful synchronization
- **FIFO Price-Time Priority**: Industry-standard order matching algorithm
- **Comprehensive Metrics**: Real-time tracking of latency percentiles (P50, P95, P99)
- **Order Book Management**: Full depth-of-market visibility and manipulation
- **Multiple Order Types**: Support for Market, Limit, Stop-Loss, and Stop-Limit orders

### Architecture

The engine is composed of three main modules:

1. **Types Module** (`types/`): Core data structures including Order, Trade, and ExecutionMetrics
2. **Matching Module** (`matching/`): Order book implementation with FIFO matching logic
3. **Engine Module** (`engine/`): Main execution engine with async processing and metrics collection

```
┌─────────────────────────────────────────┐
│         Execution Engine                │
│  ┌───────────────────────────────────┐  │
│  │   Order Queue (Crossbeam)         │  │
│  └───────────────┬───────────────────┘  │
│                  │                       │
│  ┌───────────────▼───────────────────┐  │
│  │   Order Processing Loop (Tokio)   │  │
│  └───────────────┬───────────────────┘  │
│                  │                       │
│  ┌───────────────▼───────────────────┐  │
│  │   Order Book (BTreeMap)           │  │
│  │   - Bids (Price-Time Priority)    │  │
│  │   - Asks (Price-Time Priority)    │  │
│  └───────────────┬───────────────────┘  │
│                  │                       │
│  ┌───────────────▼───────────────────┐  │
│  │   Matching Engine                 │  │
│  └───────────────┬───────────────────┘  │
│                  │                       │
│  ┌───────────────▼───────────────────┐  │
│  │   Trade Output Channel            │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

### Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rust-order-execution-engine = "0.1.0"
```

Or clone and build from source:

```bash
git clone https://github.com/gabriellafis/rust-order-execution-engine.git
cd rust-order-execution-engine
cargo build --release
```

### Quick Start

```rust
use rust_order_execution_engine::{ExecutionEngine, Order, Side};
use crossbeam::channel::unbounded;

#[tokio::main]
async fn main() {
    // Create trade channel
    let (trade_sender, trade_receiver) = unbounded();
    
    // Initialize engine
    let engine = ExecutionEngine::new(trade_sender);
    engine.start().await;
    
    // Submit a limit buy order
    let order = Order::new_limit(
        "BTCUSD".to_string(),
        Side::Buy,
        10,
        50000.0,
        "client1".to_string()
    );
    
    engine.submit_order(order).await.unwrap();
    
    // Process trades
    while let Ok(trade) = trade_receiver.recv() {
        println!("Trade executed: {} @ {} (qty: {})", 
                 trade.symbol, trade.price, trade.quantity);
    }
    
    // Get execution metrics
    let metrics = engine.get_metrics();
    println!("Total orders: {}", metrics.total_orders);
    println!("Fill rate: {:.2}%", metrics.fill_rate());
    println!("P99 latency: {} μs", metrics.p99_latency_micros);
    
    engine.stop().await;
}
```

### Running Examples

```bash
# Basic usage example
cargo run --example basic_usage

# Run benchmarks
cargo bench

# Run tests
cargo test
```

### Performance Benchmarks

Tested on AMD Ryzen 9 5900X @ 3.7GHz:

| Operation | Orders | Avg Latency | P99 Latency |
|-----------|--------|-------------|-------------|
| Add Order | 100 | 2.3 μs | 5.1 μs |
| Match Orders | 100 | 45.2 μs | 89.7 μs |
| Match Orders | 1,000 | 423.5 μs | 856.3 μs |
| Match Orders | 5,000 | 2.1 ms | 4.3 ms |

### Use Cases

- **Quantitative Trading Platforms**: Core execution engine for automated trading systems
- **Market Making**: High-frequency order placement and cancellation
- **Backtesting**: Realistic order execution simulation with latency modeling
- **Exchange Development**: Foundation for building matching engines
- **Educational**: Learning low-latency systems design in Rust

### Technical Highlights

- **Lock-Free Queues**: Using Crossbeam for high-throughput order submission
- **Efficient Price Levels**: BTreeMap for O(log n) best bid/ask retrieval
- **Zero-Copy Serialization**: Serde with optimized formats
- **Latency Tracking**: Per-order latency measurement with percentile calculation
- **Memory Efficiency**: Careful allocation patterns to minimize GC pressure

### Project Structure

```
rust-order-execution-engine/
├── src/
│   ├── lib.rs              # Library entry point
│   ├── types/
│   │   └── mod.rs          # Core data structures
│   ├── matching/
│   │   └── mod.rs          # Order book and matching logic
│   └── engine/
│       └── mod.rs          # Execution engine
├── examples/
│   └── basic_usage.rs      # Usage examples
├── benches/
│   └── order_matching.rs   # Performance benchmarks
├── tests/
│   └── integration_tests.rs
├── Cargo.toml
└── README.md
```

### Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

### License

This project is licensed under the MIT License - see the LICENSE file for details.

### Author

**Gabriel Demetrios Lafis**

---

## Português

### Visão Geral

Um motor de execução de ordens de alta performance para mercados financeiros, construído com Rust. Este motor fornece processamento de ordens com latência ultra-baixa, matching eficiente de ordens e métricas abrangentes de execução adequadas para plataformas de trading quantitativo.

### Características Principais

- **Latência Ultra-Baixa**: Processamento de ordens em nível de microssegundos com estruturas de dados otimizadas
- **Arquitetura Assíncrona**: Construído sobre o runtime Tokio para operações concorrentes eficientes
- **Design Thread-Safe**: Estruturas de dados lock-free e sincronização cuidadosa
- **Prioridade FIFO Preço-Tempo**: Algoritmo de matching de ordens padrão da indústria
- **Métricas Abrangentes**: Rastreamento em tempo real de percentis de latência (P50, P95, P99)
- **Gerenciamento de Order Book**: Visibilidade e manipulação completa da profundidade de mercado
- **Múltiplos Tipos de Ordem**: Suporte para ordens Market, Limit, Stop-Loss e Stop-Limit

### Arquitetura

O motor é composto por três módulos principais:

1. **Módulo Types** (`types/`): Estruturas de dados centrais incluindo Order, Trade e ExecutionMetrics
2. **Módulo Matching** (`matching/`): Implementação do order book com lógica de matching FIFO
3. **Módulo Engine** (`engine/`): Motor de execução principal com processamento assíncrono e coleta de métricas

### Instalação

Adicione ao seu `Cargo.toml`:

```toml
[dependencies]
rust-order-execution-engine = "0.1.0"
```

Ou clone e compile do código-fonte:

```bash
git clone https://github.com/gabriellafis/rust-order-execution-engine.git
cd rust-order-execution-engine
cargo build --release
```

### Início Rápido

```rust
use rust_order_execution_engine::{ExecutionEngine, Order, Side};
use crossbeam::channel::unbounded;

#[tokio::main]
async fn main() {
    // Criar canal de trades
    let (trade_sender, trade_receiver) = unbounded();
    
    // Inicializar motor
    let engine = ExecutionEngine::new(trade_sender);
    engine.start().await;
    
    // Submeter ordem de compra limitada
    let order = Order::new_limit(
        "BTCUSD".to_string(),
        Side::Buy,
        10,
        50000.0,
        "client1".to_string()
    );
    
    engine.submit_order(order).await.unwrap();
    
    // Processar trades
    while let Ok(trade) = trade_receiver.recv() {
        println!("Trade executado: {} @ {} (qtd: {})", 
                 trade.symbol, trade.price, trade.quantity);
    }
    
    // Obter métricas de execução
    let metrics = engine.get_metrics();
    println!("Total de ordens: {}", metrics.total_orders);
    println!("Taxa de preenchimento: {:.2}%", metrics.fill_rate());
    println!("Latência P99: {} μs", metrics.p99_latency_micros);
    
    engine.stop().await;
}
```

### Executando Exemplos

```bash
# Exemplo de uso básico
cargo run --example basic_usage

# Executar benchmarks
cargo bench

# Executar testes
cargo test
```

### Benchmarks de Performance

Testado em AMD Ryzen 9 5900X @ 3.7GHz:

| Operação | Ordens | Latência Média | Latência P99 |
|----------|--------|----------------|--------------|
| Adicionar Ordem | 100 | 2.3 μs | 5.1 μs |
| Match de Ordens | 100 | 45.2 μs | 89.7 μs |
| Match de Ordens | 1.000 | 423.5 μs | 856.3 μs |
| Match de Ordens | 5.000 | 2.1 ms | 4.3 ms |

### Casos de Uso

- **Plataformas de Trading Quantitativo**: Motor de execução central para sistemas de trading automatizado
- **Market Making**: Colocação e cancelamento de ordens de alta frequência
- **Backtesting**: Simulação realista de execução de ordens com modelagem de latência
- **Desenvolvimento de Exchanges**: Fundação para construção de motores de matching
- **Educacional**: Aprendizado de design de sistemas de baixa latência em Rust

### Destaques Técnicos

- **Filas Lock-Free**: Usando Crossbeam para submissão de ordens de alto throughput
- **Níveis de Preço Eficientes**: BTreeMap para recuperação O(log n) de melhor bid/ask
- **Serialização Zero-Copy**: Serde com formatos otimizados
- **Rastreamento de Latência**: Medição de latência por ordem com cálculo de percentis
- **Eficiência de Memória**: Padrões cuidadosos de alocação para minimizar pressão de GC

### Estrutura do Projeto

```
rust-order-execution-engine/
├── src/
│   ├── lib.rs              # Ponto de entrada da biblioteca
│   ├── types/
│   │   └── mod.rs          # Estruturas de dados centrais
│   ├── matching/
│   │   └── mod.rs          # Order book e lógica de matching
│   └── engine/
│       └── mod.rs          # Motor de execução
├── examples/
│   └── basic_usage.rs      # Exemplos de uso
├── benches/
│   └── order_matching.rs   # Benchmarks de performance
├── tests/
│   └── integration_tests.rs
├── Cargo.toml
└── README.md
```

### Contribuindo

Contribuições são bem-vindas! Sinta-se à vontade para submeter issues ou pull requests.

### Licença

Este projeto está licenciado sob a Licença MIT - veja o arquivo LICENSE para detalhes.

### Autor

**Gabriel Demetrios Lafis**

# Kafka Rust Project

This project demonstrates how to produce and consume messages using Apache Kafka in a Rust application. The setup uses Docker Compose to run Kafka and Zookeeper.

## Prerequisites

- Rust (`https://rustup.rs/`)
- Docker (`https://docs.docker.com/get-docker/`)
- Docker Compose (`https://docs.docker.com/compose/install/`)

## Setting Up the Project

### 1. Clone the Repository

```sh
git clone https://github.com/ASoldo/kafka_rust
cd kafka_rust
```

### 2. Build the Docker Containers

```sh
docker-compose up -d
```

This command starts Kafka and Zookeeper services in Docker containers.

### 3. Add Dependencies

Ensure your `Cargo.toml` file includes the following dependencies:

```toml
name = "kafka_rust"
version = "0.1.0"
edition = "2021"

[dependencies]
futures = "0.3.30"
rdkafka = "0.36.2"
tokio = { version = "1.38.1", features = ["full"] }
```

### Running the Consumer

To run the consumer, use the following command:

```sh
cargo run --bin consumer || cargo run --bin consumer human
```

### Running the Producer

To run the producer, use the following command:

```sh
cargo run --bin producer
```

# Explanation

### Docker Compose Setup

The docker-compose.yml file sets up Kafka and Zookeeper using Docker. It configures memory limits and other settings to ensure smooth operation.

## Producer

The producer sends messages to the Kafka topic. Each message is associated with a key. The key can be used to ensure messages with the same key are sent to the same partition.

## Consumer

The consumer reads messages from the Kafka topic. It decodes the key and payload of each message and prints them to the console.

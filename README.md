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
actix-web = "4.8.0"
futures = "0.3.30"
rdkafka = "0.36.2"
serde = { version = "1.0.204", features = ["derive"] }
serde_json = "1.0.120"
tokio = { version = "1.38.1", features = ["full"] }
uuid = { version = "1.10.0", features = ["v4"]}
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

### Running the Actix API

To run the Actix API, use the following command:

```sh
cargo run --bin kafka_rust
```

### Curl API Example:

Here are the curl commands to test the CRUD operations:

1. Produce a Message
   To send a message to Kafka, use the following command:

```sh
curl -X POST http://127.0.0.1:8080/produce -H "Content-Type: application/json" -d '{"message": "Hello, Kafka!", "key": "some_key"}'
```

2. Get a Message
   To retrieve a message by its ID:

```sh
curl -X GET http://127.0.0.1:8080/messages/<message_id>
```

Replace <message_id> with the actual message ID returned when you produced the message.

3. Update a Message
   To update a message by its ID:

```sh
curl -X PUT http://127.0.0.1:8080/messages/<message_id> -H "Content-Type: application/json" -d '{"message": "Updated message", "key": "new_key"}'
```

Replace <message_id> with the actual message ID.

4. Delete a Message
   To delete a message by its ID:

```sh
curl -X DELETE http://127.0.0.1:8080/messages/<message_id>
```

Replace <message_id> with the actual message ID.

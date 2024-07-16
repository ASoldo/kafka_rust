use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use tokio::runtime::Runtime;

async fn produce_message(broker: &str, topic: &str, message: &str, key: &str) {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", broker)
        .create()
        .expect("Producer creation error");

    let delivery_status = producer
        .send(
            FutureRecord::to(topic).payload(message).key(key),
            Timeout::Never,
        )
        .await;

    println!("Delivery status: {:?}", delivery_status);
}

fn main() {
    let runtime = Runtime::new().unwrap();
    runtime.block_on(async {
        produce_message("localhost:9092", "test-topic", "Hello, Kafka!", "some_key").await;
        produce_message("localhost:9092", "test-topic", "Yo, Kafka!", "some_key").await;
        produce_message(
            "localhost:9092",
            "test-topic",
            "Another Kafka!",
            "another_kafka",
        )
        .await;
        produce_message(
            "localhost:9092",
            "test-topic",
            "Bye, Kafka!",
            "another_kafka",
        )
        .await;
    });
}

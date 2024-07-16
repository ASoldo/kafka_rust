use futures::stream::StreamExt;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::util::get_rdkafka_version;
use std::env;
use tokio::runtime::Runtime;

fn decode_bytes(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}

async fn consume_messages(broker: &str, group_id: &str, topic: &str, human_readable: bool) {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", broker)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .create()
        .expect("Consumer creation failed");

    consumer
        .subscribe(&[topic])
        .expect("Can't subscribe to specified topic");

    let mut message_stream = consumer.stream();

    while let Some(message) = message_stream.next().await {
        match message {
            Err(e) => println!("Kafka error: {}", e),
            Ok(m) => {
                if human_readable {
                    let payload = m.payload().unwrap_or(b"");
                    let key = m.key().unwrap_or(b"");
                    println!("key: '{}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                             decode_bytes(key), decode_bytes(payload), m.topic(), m.partition(), m.offset(), m.timestamp());
                } else {
                    let payload = m.payload().unwrap_or(b"");
                    let key = m.key().unwrap_or(b"");
                    println!("key: '{:?}', payload: '{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                             key, payload, m.topic(), m.partition(), m.offset(), m.timestamp());
                }
            }
        }
    }
}

fn main() {
    let (version_n, version_s) = get_rdkafka_version();
    println!("rd_kafka_version: 0x{:08x}, {}", version_n, version_s);

    let args: Vec<String> = env::args().collect();
    let human_readable = args.get(1).map_or(false, |arg| arg == "human");

    let runtime = Runtime::new().unwrap();
    runtime.block_on(async {
        consume_messages("localhost:9092", "test_group", "test-topic", human_readable).await;
    });
}

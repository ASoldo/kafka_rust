use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tokio::sync::broadcast;
use uuid::Uuid;

/// Structure representing a request to produce a message.
#[derive(Deserialize)]
struct ProduceRequest {
    message: String,
    key: String,
}

/// Structure representing a message with an ID, key, and content.
#[derive(Serialize, Deserialize, Clone, Debug)]
struct Message {
    id: String,
    key: String,
    message: String,
}

/// Application state holding the Kafka producer, stored messages, and a broadcaster for SSE.
struct AppState {
    producer: FutureProducer,
    messages: Mutex<Vec<Message>>,
    broadcaster: broadcast::Sender<Message>,
}

/// Produces a message to Kafka and stores it in the application state.
///
/// # Arguments
///
/// * `data` - Application state.
/// * `req` - JSON payload containing the message and key.
///
/// # Returns
///
/// A JSON response indicating success or failure.
async fn produce_message(
    data: web::Data<AppState>,
    req: web::Json<ProduceRequest>,
) -> impl Responder {
    let record = FutureRecord::to("test-topic")
        .payload(&req.message)
        .key(&req.key);

    match data.producer.send(record, Timeout::Never).await {
        Ok(_) => {
            let id = Uuid::new_v4().to_string();
            let new_message = Message {
                id: id.clone(),
                key: req.key.clone(),
                message: req.message.clone(),
            };
            data.messages.lock().unwrap().push(new_message.clone());

            let _ = data.broadcaster.send(new_message);

            HttpResponse::Ok().json(format!(
                "Message '{}' with key '{}' sent successfully. ID: {}",
                req.message, req.key, id
            ))
        }
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Failed to send message: {:?}", e))
        }
    }
}

/// Retrieves a message by its ID.
///
/// # Arguments
///
/// * `data` - Application state.
/// * `message_id` - The ID of the message to retrieve.
///
/// # Returns
///
/// A JSON response containing the message or an error message if not found.
async fn get_message(data: web::Data<AppState>, message_id: web::Path<String>) -> impl Responder {
    let messages = data.messages.lock().unwrap();
    let id = message_id.clone();
    match messages.iter().find(|msg| msg.id == id) {
        Some(message) => HttpResponse::Ok().json(message),
        None => HttpResponse::NotFound().body("Message not found"),
    }
}

/// Updates a message by its ID.
///
/// # Arguments
///
/// * `data` - Application state.
/// * `message_id` - The ID of the message to update.
/// * `req` - JSON payload containing the new message and key.
///
/// # Returns
///
/// A JSON response indicating success or failure.
async fn update_message(
    data: web::Data<AppState>,
    message_id: web::Path<String>,
    req: web::Json<ProduceRequest>,
) -> impl Responder {
    let mut messages = data.messages.lock().unwrap();
    let id = message_id.clone();
    match messages.iter_mut().find(|msg| msg.id == id) {
        Some(message) => {
            message.key = req.key.clone();
            message.message = req.message.clone();

            let record = FutureRecord::to("test-topic")
                .payload(&message.message)
                .key(&message.key);

            match data.producer.send(record, Timeout::Never).await {
                Ok(_) => {
                    let _ = data.broadcaster.send(message.clone());

                    HttpResponse::Ok().json(format!(
                        "Message '{}' with key '{}' updated successfully",
                        req.message, req.key
                    ))
                }
                Err(e) => HttpResponse::InternalServerError()
                    .body(format!("Failed to update message: {:?}", e)),
            }
        }
        None => HttpResponse::NotFound().body("Message not found"),
    }
}

/// Deletes a message by its ID.
///
/// # Arguments
///
/// * `data` - Application state.
/// * `message_id` - The ID of the message to delete.
///
/// # Returns
///
/// A response indicating success or failure.
async fn delete_message(
    data: web::Data<AppState>,
    message_id: web::Path<String>,
) -> impl Responder {
    let mut messages = data.messages.lock().unwrap();
    let id = message_id.clone();
    if let Some(pos) = messages.iter().position(|msg| msg.id == id) {
        messages.remove(pos);
        HttpResponse::Ok().body("Message deleted")
    } else {
        HttpResponse::NotFound().body("Message not found")
    }
}

/// Handles Server-Side Events (SSE) connections.
///
/// # Arguments
///
/// * `data` - Application state.
///
/// # Returns
///
/// A streaming response sending new messages as they are produced.
async fn sse(data: web::Data<AppState>) -> impl Responder {
    let mut rx = data.broadcaster.subscribe();

    HttpResponse::Ok()
        .content_type("text/event-stream")
        .streaming(
            async_stream::stream! {
                while let Ok(message) = rx.recv().await {
                    yield Ok::<_, actix_web::Error>(web::Bytes::from(format!("data: {:?}\n\n", message)));
                }
            },
        )
}

/// Main function to start the Actix web server.
///
/// Initializes the Kafka producer and application state, and sets up the HTTP routes.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .create()
        .expect("Producer creation error");

    let (tx, _rx) = broadcast::channel(100);

    let app_state = web::Data::new(AppState {
        producer,
        messages: Mutex::new(Vec::new()),
        broadcaster: tx,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/produce", web::post().to(produce_message))
            .route("/messages/{id}", web::get().to(get_message))
            .route("/messages/{id}", web::put().to(update_message))
            .route("/messages/{id}", web::delete().to(delete_message))
            .route("/events", web::get().to(sse))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Deserialize)]
struct ProduceRequest {
    message: String,
    key: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct Message {
    id: String,
    key: String,
    message: String,
}

struct AppState {
    producer: FutureProducer,
    messages: Mutex<Vec<Message>>,
}

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
            data.messages.lock().unwrap().push(new_message);

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

async fn get_message(data: web::Data<AppState>, message_id: web::Path<String>) -> impl Responder {
    let messages = data.messages.lock().unwrap();
    let id = message_id.clone();
    match messages.iter().find(|msg| msg.id == id) {
        Some(message) => HttpResponse::Ok().json(message),
        None => HttpResponse::NotFound().body("Message not found"),
    }
}

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
                Ok(_) => HttpResponse::Ok().json(format!(
                    "Message '{}' with key '{}' updated successfully",
                    req.message, req.key
                )),
                Err(e) => HttpResponse::InternalServerError()
                    .body(format!("Failed to update message: {:?}", e)),
            }
        }
        None => HttpResponse::NotFound().body("Message not found"),
    }
}

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .create()
        .expect("Producer creation error");

    let app_state = web::Data::new(AppState {
        producer,
        messages: Mutex::new(Vec::new()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/produce", web::post().to(produce_message))
            .route("/messages/{id}", web::get().to(get_message))
            .route("/messages/{id}", web::put().to(update_message))
            .route("/messages/{id}", web::delete().to(delete_message))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

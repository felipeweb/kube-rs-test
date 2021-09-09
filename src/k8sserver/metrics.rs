use actix_web::{
    get, middleware,
    web::{self, Data},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};

use crate::manager::Manager;
use tracing::{debug, error, info, trace, warn};
use prometheus::{Encoder, TextEncoder};

#[get("/metrics")]
pub async fn metrics(c: Data<Manager>, _req: HttpRequest) -> impl Responder {
    let metrics = c.metrics();
    let encoder = TextEncoder::new();
    let mut buffer = vec![];
    encoder.encode(&metrics, &mut buffer).unwrap();
    HttpResponse::Ok().body(buffer)
}
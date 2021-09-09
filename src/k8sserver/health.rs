use actix_web::{
    get, middleware,
    web::{self, Data},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Health {
    status: String,
}

#[get("/health")]
pub async fn health(_: HttpRequest) -> impl Responder {
    HttpResponse::Ok().json(Health {
        status: "ok".to_string(),
    })
}

use actix_web::{get, App, HttpServer, Responder};
use tokio::time::{sleep, Duration};

#[get("/")]
async fn greet() -> impl Responder {
    get_from_db().await;
    update_cache().await;
    "Hello World!"
}

async fn get_from_db() -> () {
    sleep(Duration::from_millis(250)).await;
}

async fn update_cache() -> () {
    sleep(Duration::from_millis(75)).await;
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(greet)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

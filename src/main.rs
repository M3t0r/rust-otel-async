use actix_web::{get, post, App, HttpServer, Responder};
use tokio::time::{sleep, Duration};

#[get("/")]
async fn greet() -> impl Responder {
    let _ms_resp = awc::Client::default()
        .post("http://127.0.0.1:8080/microservice")
        .send()
        .await
        .expect("requesting sub-service")
        .body()
        .await
        .expect("reading response from sub-service");

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

#[post("/microservice")]
async fn microservice() -> impl Responder {
    upsert_into_db().await;
    "Hello microservice!"
}

async fn upsert_into_db() -> () {
    sleep(Duration::from_millis(375)).await;
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(greet).service(microservice))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

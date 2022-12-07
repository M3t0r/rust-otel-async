use actix_web::{get, post, App, HttpServer, Responder};
use tokio::time::{sleep, Duration};

use opentelemetry::{
    sdk::{trace::Sampler, Resource},
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;

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

fn setup_tracing() -> () {
    // prepare gRPC metadata (HTTP headers) with our honeycomb key
    let mut headers = tonic::metadata::MetadataMap::with_capacity(1);
    headers.insert(
        "x-honeycomb-team",
        std::env::var("HONEYCOMB_TOKEN")
            .expect("retreiving HONEYCOMB_TOKEN")
            .parse()
            .expect("parsing HONEYCOMB_TOKEN"),
    );

    // create a gRPC OTLP exporter
    let tracing_exporter = opentelemetry_otlp::new_exporter()
        .tonic() // gRPC
        .with_endpoint("https://api.honeycomb.io")
        .with_metadata(headers);

    // setup the otlp pipeline with batching and sampler
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(tracing_exporter)
        .with_trace_config(
            opentelemetry::sdk::trace::config()
                .with_sampler(Sampler::AlwaysOn)
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.name", env!("CARGO_PKG_NAME")),
                    KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                ])),
        )
        .install_batch(opentelemetry::runtime::TokioCurrentThread)
        .expect("tracing pipeline installation");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    setup_tracing();
    HttpServer::new(|| {
        App::new()
            .wrap(actix_web_opentelemetry::RequestTracing::new())
            .service(greet)
            .service(microservice)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

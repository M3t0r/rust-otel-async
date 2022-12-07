use actix_web::{get, post, App, HttpServer, Responder};
use tokio::time::{sleep, Duration};

use actix_web_opentelemetry::ClientExt;
use opentelemetry::{
    global::tracer,
    sdk::{trace::Sampler, Resource},
    trace::{mark_span_as_active, Tracer},
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;

#[get("/")]
async fn greet() -> impl Responder {
    let _ms_resp = awc::Client::default()
        .post("http://127.0.0.1:8080/microservice")
        .trace_request()
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
    let _span_guard = mark_span_as_active(tracer("mainservice").start("get_from_db"));

    sleep(Duration::from_millis(250)).await;
}

async fn update_cache() -> () {
    let _span_guard = mark_span_as_active(tracer("mainservice").start("update_cache"));

    sleep(Duration::from_millis(75)).await;
}

#[post("/microservice")]
async fn microservice() -> impl Responder {
    upsert_into_db().await;
    "Hello microservice!"
}

async fn upsert_into_db() -> () {
    let _span_guard = mark_span_as_active(tracer("microservice").start("upsert_into_db"));

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

    // add propagation to connect spans with remote systems
    opentelemetry::global::set_text_map_propagator(
        opentelemetry::sdk::propagation::TraceContextPropagator::new(),
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
                .with_sampler(Sampler::ParentBased(std::boxed::Box::new(
                    Sampler::AlwaysOn,
                )))
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

mod config;
mod crypt;
mod ctx;
mod error;
mod log;
mod model;
mod util;
mod web;

pub mod _dev_utils;

pub use self::error::{Error, Result};
pub use config::config;

use std::net::SocketAddr;

use axum::{
    extract::{Path, Query},
    http::{Method, Uri},
    middleware,
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
//use ctx::Ctx;
use model::ModelManager;
use serde::Deserialize;
use serde_json::json;
use tower_cookies::CookieManagerLayer;
use tracing::info;
use web::{
    mw_auth::{mw_ctx_require, CtxW},
    rpc,
};

#[tokio::main]
async fn main() -> Result<()> {
    let format = tracing_subscriber::fmt::format()
        .with_level(false) // don't include levels in formatted output
        .with_target(false) // don't include targets
        .with_thread_ids(false) // include the thread ID of the current thread
        .with_thread_names(false) // include the name of the current thread
        .compact(); // use the `Compact` formatting style.

    tracing_subscriber::fmt().event_format(format).init();

    _dev_utils::init_dev().await;

    let mm = ModelManager::new().await?;

    let routes_rpc = rpc::routes(mm.clone()).route_layer(middleware::from_fn(mw_ctx_require));

    let routes_all = Router::new()
        .merge(routes_hello())
        .merge(web::routes_login::routes(mm.clone()))
        .nest("/api", routes_rpc)
        .layer(middleware::map_response(main_response_mapper))
        .layer(middleware::from_fn_with_state(
            mm.clone(),
            web::mw_auth::mw_ctx_resolve,
        ))
        .layer(CookieManagerLayer::new())
        .fallback_service(web::routes_static::serve_dir(&config().WEB_FOLDER));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    axum::Server::bind(&addr)
        .serve(routes_all.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn main_response_mapper(
    ctx: Option<CtxW>,
    uri: Uri,
    req_method: Method,
    res: Response,
) -> Response {
    info!("->> {:<12} - main_response_mapper", "RES_MAPPER");

    let uuid = uuid::Uuid::new_v4();

    let service_error = res.extensions().get::<Error>();
    let client_status_error = service_error.map(|e| e.client_status_and_error());

    let error_response = client_status_error.as_ref().map(|(status, client_error)| {
        let client_error_body = json!({
            "error": client_error.as_ref(),
            "uuid": uuid.to_string(),
        });

        (*status, Json(client_error_body)).into_response()
    });

    let client_error = client_status_error.unzip().1;
    let _ = log::log_request(
        uuid.to_string(),
        req_method.to_string(),
        uri,
        ctx.map(|ctx| ctx.0),
        service_error,
        client_error,
    )
    .await;

    println!();

    error_response.unwrap_or(res)
}

fn routes_hello() -> Router {
    Router::new()
        .route("/hello", get(handler_hello))
        .route("/hello2/:name", get(handler_hello2))
}

#[derive(Debug, Deserialize)]
struct HelloParams {
    name: Option<String>,
}

async fn handler_hello(Query(params): Query<HelloParams>) -> impl IntoResponse {
    println!("->> {:<12} - handler_hello - {params:?}", "HANDLER");

    let name = params.name.as_deref().unwrap_or("World");

    Html(format!("hello {name}!!!"))
}

async fn handler_hello2(Path(name): Path<String>) -> impl IntoResponse {
    println!("->> {:<12} - handler_hello - {name:?}", "HANDLER");

    Html(format!("hello {name}!!!"))
}

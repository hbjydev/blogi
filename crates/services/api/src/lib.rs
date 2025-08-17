use std::{net::SocketAddr, sync::Arc, time::Duration};

use anyhow::Result;
use axum::{body::HttpBody, extract::MatchedPath, response::Response, routing::get, Router};
use http::Request;
use state::AppState;
use tokio::net::TcpListener;
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};
use tracing::Span;

mod state;
mod handlers;

pub async fn start(
    bind_addr: SocketAddr,
    _datastore: Box<dyn blogi_db::Datastore>,
) -> Result<()> {
    let state = AppState {
        db: Arc::new(_datastore),
    };

    let router = Router::new()
        .route("/xrpc/_health", get(handlers::health::xrpc_health))

        .route("/xrpc/moe.hayden.blogi.actor.getProfile", get(handlers::actor::get_actor))
        .route("/xrpc/moe.hayden.blogi.actor.getProfiles", get(handlers::actor::list_actors))

        .with_state(state)
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let uri = request.uri();

                    let matched_path = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str);

                    tracing::info_span!(
                        "http.route",
                        http.request.method = ?request.method(),
                        url.path = uri.path(),
                        url.scheme = uri.scheme_str(),
                        network.protocol.name = "http",
                        server.port = uri.port_u16(),
                        url.query = uri.query().unwrap_or(""),
                        http.route = matched_path,
                        http.response.size = tracing::field::Empty,
                        http.response.status_code = tracing::field::Empty,
                    )
                })
                .on_response(|response: &Response, latency: Duration, span: &Span| {
                    {
                        let size_hint = response.size_hint();
                        span.record("http.response.size", size_hint.exact().unwrap_or(size_hint.lower()));
                    }
                    span.record("http.response.status_code", response.status().as_u16());

                    tracing::info!("request processed in {:?}ms", latency.as_millis());
                })
        );

    let listener = TcpListener::bind(bind_addr).await?;
    tracing::info!("listening on {}", bind_addr);
    Ok(axum::serve(listener, router).await?)
}

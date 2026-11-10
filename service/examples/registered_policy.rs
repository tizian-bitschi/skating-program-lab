//! Explicit browser acceptance fixture; the normal service never imports this policy.
#[path = "../tests/support/mod.rs"]
mod support;

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3156")
        .await
        .expect("bind registered-policy acceptance fixture");
    axum::serve(
        listener,
        skating_planning_service::app_with_catalog(support::catalog()),
    )
    .await
    .expect("registered-policy acceptance fixture");
}

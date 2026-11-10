#[tokio::main]
async fn main() {
    let address = std::env::var("C2_LISTEN").unwrap_or_else(|_| "127.0.0.1:3000".into());
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .expect("bind local planning service");
    println!("C2 Mixer: http://{address}");
    axum::serve(listener, skating_planning_service::app())
        .await
        .expect("planning service");
}

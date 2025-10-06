mod cors;
mod routes;

use std::error::Error;
use std::net::SocketAddr;

use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let state = cors::build_state().expect("valid CORS configuration");

    let addr: SocketAddr = "127.0.0.1:5003".parse().unwrap();
    let listener = TcpListener::bind(addr).await?;

    println!("Hyper example running on http://{addr}");

    loop {
        let (stream, _) = listener.accept().await?;
        let state = state.clone();
        let io = TokioIo::new(stream);

        tokio::spawn(async move {
            let service = cors::middleware::BunnerCors::new(state.cors.clone(), routes::router(state));

            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                eprintln!("connection error: {err}");
            }
        });
    }
}

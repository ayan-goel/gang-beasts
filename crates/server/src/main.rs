use brawler_server::{WorkerPool, api_for_origin};
use std::{env, error::Error, net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;

fn setting<T>(key: &str, default: T) -> Result<T, Box<dyn Error>>
where
    T: std::str::FromStr,
    T::Err: Error + 'static,
{
    match env::var(key) {
        Ok(value) => value
            .parse()
            .map_err(|e| format!("invalid {key}: {e}").into()),
        Err(env::VarError::NotPresent) => Ok(default),
        Err(e) => Err(e.into()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let address = setting("BRAWLER_LISTEN", SocketAddr::from(([127, 0, 0, 1], 8080)))?;
    let workers = setting("BRAWLER_WORKERS", 1usize)?;
    let origin = setting("BRAWLER_ORIGIN", "http://127.0.0.1:5173".to_owned())?;
    let app = api_for_origin(Arc::new(WorkerPool::new(workers)?), &origin)?;
    let listener = TcpListener::bind(address).await?;
    println!(
        "Skyline server listening on http://{}",
        listener.local_addr()?
    );
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        if let Err(e) = tokio::signal::ctrl_c().await {
            eprintln!("shutdown signal error: {e}");
        }
    })
    .await?;
    Ok(())
}

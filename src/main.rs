mod extract;
mod models;
use extract::YoutubeExtractor;
use tracing::{info, Level};
use tracing_subscriber::{fmt, EnvFilter};

#[async_std::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("rust_youtube_scraper=debug")
        .init();

    let youtube_extractor = YoutubeExtractor::new();
    let idk_return = youtube_extractor.extract("cwBoUuy4nGc", false).await;
}

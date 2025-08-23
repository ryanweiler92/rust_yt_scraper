mod extract;
mod models;
use extract::YoutubeExtractor;

#[async_std::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("rust_youtube_scraper=warn")
        .init();

    let youtube_extractor = YoutubeExtractor::new();
    let data = youtube_extractor.extract("ztWCPu8f5-Q").await;
}

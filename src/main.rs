mod extract;
mod models;
use extract::YoutubeExtractor;

#[async_std::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let youtube_extractor = YoutubeExtractor::new();
    let _data = youtube_extractor.extract("https://www.youtube.com/watch?v=gXyFe7jcufE").await;
}

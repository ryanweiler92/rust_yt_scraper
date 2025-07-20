mod extract;
mod models;

use extract::YoutubeExtractor;


#[async_std::main]
async fn main() {
    let youtube_extractor = YoutubeExtractor::new();
    let idk_return = youtube_extractor.extract("cwBoUuy4nGc").await;
}

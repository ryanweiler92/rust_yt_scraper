mod extract;
mod models;

use extract::YoutubeExtractor;

use async_std::fs;
use regex::Regex;
use serde_json::Value;



#[async_std::main]
async fn main() {
    let youtube_extractor = YoutubeExtractor::new();
    let idk_return = youtube_extractor.extract("bHD2aUFDI1E").await;
}

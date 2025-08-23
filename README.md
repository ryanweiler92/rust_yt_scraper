# yt-scraper

![Cover](assets/cover.png)

A Rust library for scraping YouTube video metadata and comments.

## Features

- Extract complete video metadata (title, channel, views, likes, description, thumbnails, etc.)
- Scrape comment threads with hierarchical reply structure

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
yt-scraper = "0.1.0"
```

## Usage

### Basic Example

```rust
use yt_scraper::{YoutubeExtractor, VideoInfo, Comment};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let extractor = YoutubeExtractor::new();
    
    // Extract video info and comments
    let (video_info, comments) = extractor.extract("dQw4w9WgXcQ", false).await?;
    
    println!("Title: {}", video_info.title);
    println!("Channel: {}", video_info.channel_name);
    println!("Views: {}", video_info.view_count);
    println!("Comments: {}", comments.len());
    
    Ok(())
}
```

## License

MIT License

## Author

Ryan Weiler (ryanweiler92@gmail.com)
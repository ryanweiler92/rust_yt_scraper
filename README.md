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

## Example Video Metadata Extraction
```json
{
  "title": "Manchester City v. Tottenham Hotspur | PREMIER LEAGUE HIGHLIGHTS | 8/23/2025 | NBC Sports",
  "channel": "NBC Sports",
  "channel_id": "UCqZQlzSHbVJrwrn5XvzrzcA",
  "description": "Look back on full-match highlights from Manchester City's Matchweek 2 showdown with Tottenham Hotspur at the Etihad. #NBCSports #PremierLeague #TottenhamHotspur #ManchesterCity\n» Subscribe to NBC Sports: https://www.youtube.com/nbcsports?sub...\n» Watch Live Sports on NBC.com: https://www.nbc.com/live-upcoming\n» Get Premier League news on NBC Sports: https://nbcsports.com/soccer/premier-...\n\nWant more Premier League? Check out Peacock: https://peacocktv.smart.link/v82e9dl56\n\nNBC Sports Group serves sports fans 24/7 with premier live events, insightful studio shows, and compelling original programming. NBC Sports is an established leader in the sports media landscape with an unparalleled collection of sports properties that include the Olympics, NFL, Premier League, NASCAR, PGA TOUR, the Kentucky Derby, Tour de France and many more.\n\nSubscribe to our channel for the latest sporting news and highlights!\n\nThe Premier League across NBC Sports Group launched in 2013 with their biggest and broadest programming commitment to-date in the United States. With live multi-platform coverage of all 380 games, analysis from best-in-class talent and extensive surrounding coverage all week long, NBC Sports Group has become the ultimate destination for new and existing Premier League fans.\n\nThe Premier League maintains strong and consistent reach across NBC, USA Network, CNBC, and NBC Sports Group’s live streaming products, led by the biggest stars and most prestigious teams in the world.\n\nVisit NBC Sports: https://www.nbcsports.com\nFind NBC Sports on Facebook:   / nbcsports  \nFollow NBC Sports on Twitter:   / nbcsports  \nFollow NBC Sports on Instagram:   / nbcsports  \n\nhttps://www.nbcsports.com/nfl/sunday-...\nhttps://nbcsports.com/motors/nascar\nhttps://nbcsports.com/soccer/premier-...\n\nManchester City v. Tottenham Hotspur | PREMIER LEAGUE HIGHLIGHTS | 8/23/2025 | NBC Sports\n   / nbcsports  ",
  "yt_id": "",
  "views": 799831,
  "comment_count": 821,
  "like_count": 10464,
  "video_thumbnail": "",
  "upload_date": "Aug 23, 2025",
  "channel_thumbnail": "https://yt3.ggpht.com/aW4xHE7eoBS5B8HOEGMdaizXSn6LzfYMp9SrESfiR1Czs9GnRnnL0znnUJezjTymMUdk_PBGvak=s176-c-k-c0x00ffffff-no-rj"
}
```

## Example Comment Extraction
```json
  {
    "comment_id": "Ugxw-HYup3GaO5MKe8l4AaABAg.AMANImk0A3SAMAW3Sfkbt9",
    "channel_id": "UCmciGGIsVksCVi8n9I1-hNQ",
    "video_id": "ztWCPu8f5-Q",
    "display_name": "@Luke86811",
    "user_verified": false,
    "thumbnail": "https://yt3.ggpht.com/Uyccu0-dBlsFUSOmKzcq20xqAmut9kfDoencSBkpKZGmSb2Kv87zGrbqQzCxmdwNdGWFR4NLGA=s88-c-k-c0x00ffffff-no-rj",
    "content": "Always have been. Fantastic win for Spurs",
    "published_time": "21 hours ago",
    "like_count": 9,
    "reply_count": 0,
    "comment_level": 1,
    "reply_to": "Ugxw-HYup3GaO5MKe8l4AaABAg",
    "reply_order": 9
  }
```

## Usage

### Basic Example

```rust
use yt_scraper::{YoutubeExtractor, VideoInfo, Comment};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let extractor = YoutubeExtractor::new();
    
    // Extract video info and comments
    let (video_info, comments) = extractor.extract("dQw4w9WgXcQ").await?;
    
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
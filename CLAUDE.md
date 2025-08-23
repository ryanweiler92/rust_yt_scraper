# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview
This is a Rust YouTube scraper library that extracts video metadata and comments from YouTube videos. The project consists of:

- **Library crate**: `youtube_scraper` for external use
- **Binary crate**: Test runner that demonstrates the scraping functionality
- **Core extractor**: `YoutubeExtractor` that handles HTML parsing and API interactions
- **Comment system**: Hierarchical comment extraction with replies support
- **Models**: Structured data types for video information and comments

## Common Commands

### Build and Development
```bash
cargo check          # Fast syntax and type checking
cargo build          # Build the project
cargo build --release # Build optimized release version
cargo run            # Run the main binary (demonstrates scraping)
cargo test           # Run tests
```

### Library Usage
The project exposes `YoutubeExtractor` as the main API:
```rust
let extractor = YoutubeExtractor::new();
let (video_info, comments) = extractor.extract("video_id", false).await?;
```

## Architecture

### Core Components
- **YoutubeExtractor** (`src/extract/youtube_extractor.rs`): Main scraper class that handles:
  - HTML parsing and JSON extraction from YouTube pages
  - Video metadata extraction (title, channel, views, likes, etc.)
  - Configuration parsing (ytcfg, ytInitialData)
  - Coordinate comment extraction workflow

- **Comment extraction** (`src/extract/comment_extract.rs`): Handles:
  - Comment thread parsing from YouTube's internal API
  - Reply extraction and hierarchical structuring
  - Continuation token management for pagination
  - Comment content normalization

### Data Models (`src/models/models.rs`)
- **VideoInfo**: Complete video metadata (title, channel, views, thumbnails, etc.)
- **Comment**: Full comment structure with threading info (level, reply_to, reply_order)
- **CommentContent**: Base comment data without threading metadata

### Module Structure
```
src/
├── lib.rs              # Library exports
├── main.rs             # Test binary
├── extract/
│   ├── youtube_extractor.rs  # Core scraping logic
│   ├── comment_extract.rs    # Comment-specific extraction
│   ├── helper.rs            # Utility functions
│   └── error_msgs.rs        # Error types
└── models/
    └── models.rs            # Data structures
```

## Implementation Notes

### YouTube Data Extraction
The scraper works by:
1. Fetching the YouTube watch page HTML
2. Extracting embedded JSON data (`ytInitialData`, `ytcfg`)
3. Parsing video metadata from the JSON structure
4. Making additional API calls for comments using internal YouTube endpoints

### Comment Threading
Comments are structured hierarchically:
- `comment_level`: 0 for top-level comments, 1+ for replies
- `reply_to`: Parent comment ID for threaded conversations
- `reply_order`: Position within reply thread

### Error Handling
Custom error types in `error_msgs.rs` handle YouTube-specific failures.

### Logging
Uses `tracing` crate for structured logging. Set `RUST_LOG=rust_youtube_scraper=debug` for detailed output.


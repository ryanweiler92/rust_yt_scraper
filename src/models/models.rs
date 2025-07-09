use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct VideoInfo {
    pub title: String,
    pub channel: String,
    pub channel_id: String,
    pub description: String,
    pub yt_id: String,
    pub views: u64,
    pub comment_count: u64,
    pub like_count: u64,
    pub video_thumbnail: String,
    pub upload_date: String,
    pub channel_thumbnail: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Comment {
    pub comment_id: String,
    pub channel_id: String,
    pub video_id: String,
    pub display_name: String,
    pub user_verified: bool,
    pub thumbnail: String,
    pub content: String,
    pub published_time: String,
    pub like_count: String,
    pub reply_count: String,
    pub comment_level: u8,
    pub reply_to: String,
    pub reply_order: i32,
}
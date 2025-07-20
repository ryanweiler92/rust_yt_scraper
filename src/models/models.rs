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
    pub like_count: i32,
    pub reply_count: i32,
    pub comment_level: i32,
    pub reply_to: String,
    pub reply_order: i32,
}

impl Comment {
    pub fn from_comment_content(
        content: CommentContent,
        comment_level: i32,
        reply_to: String,
        reply_order: i32,
    ) -> Self {
        Self {
            comment_id: content.comment_id,
            channel_id: content.channel_id,
            display_name: content.display_name,
            user_verified: content.user_verified,
            thumbnail: content.thumbnail,
            content: content.content,
            published_time: content.published_time,
            like_count: content.like_count,
            reply_count: content.reply_count,
            video_id: content.video_id,
            comment_level,
            reply_to,
            reply_order,
        }

    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommentContent {
    pub comment_id: String,
    pub channel_id: String,
    pub video_id: String,
    pub display_name: String,
    pub user_verified: bool,
    pub thumbnail: String,
    pub content: String,
    pub published_time: String,
    pub like_count: i32,
    pub reply_count: i32,
}
use async_std::fs;
use serde_json::{json, Value};
use base64::{Engine as _, engine::general_purpose};
use reqwest;
use std::collections::HashMap;
use percent_encoding::percent_decode_str;

use crate::models::Comment;
use helper::string_to_bool;


use super::{helper, YoutubeExtractor};
use super::error_msgs::YoutubeError;

impl YoutubeExtractor {

    pub async fn comment_data_to_json(&self, data: &Vec<Comment>) -> Result<String, Box<dyn std::error::Error>> {
        let string_json = serde_json::to_string_pretty(data)?;
        fs::write("COMMENT.json", string_json).await?;
        Ok("Successfully wrote comments to COMMENT.json".to_string())
    }
    pub async fn comment_extractor(&self, data: &Value) -> Option<Vec<Comment>> {
        let mut comments: Vec<Comment> = Vec::new();

        let comment_content_list_test = data
            .get("frameworkUpdates")?
            .get( "entityBatchUpdate")?
            .get("mutations")?;

        let comment_test_string_pretty = serde_json::to_string_pretty(&comment_content_list_test).unwrap_or_default();

        fs::write("content_test.json", comment_test_string_pretty).await;

        let comment_content_list_actual = data
            .get("frameworkUpdates")?
            .get( "entityBatchUpdate")?
            .get("mutations")?
            .as_array()?;

        println!("comment_content_list_actual length == {}", comment_content_list_actual.len());

        for (index, comment_content) in comment_content_list_actual.iter().enumerate() {
            let author_info_json = match comment_content
                .get("payload")
                .and_then(|p| p.get("commentEntityPayload"))
                .and_then(|c| c.get("author")) {
                Some(author_json) => author_json,
                None => {
                    println!("The comment of index {} does not contain an author block. Skipping..", index);
                    continue
                }
            };

            let comment_properties_json = match comment_content
                .get("payload")
                .and_then(|p| p.get("commentEntityPayload"))
                .and_then(|c| c.get("properties")) {
                Some(properties_json) => properties_json,
                None => {
                    println!("Index {} does not include a properties block", index);
                    continue
                }
            };

            let entity_key = match self.get_text_from_path(&comment_content, &["entityKey"]) {
                Some(key) => key,
                None => {
                    println!("Failed to acquire entity key for comment # {}.", index);
                    continue
                }
            };

            let channel_id = match self.get_text_from_path(&author_info_json, &["channelId"]) {
                Some(key) => key,
                None => {
                    println!("Failed to get channel id from index: {}", index);
                    "MISSING_CHANNEL_ID".to_string()
                }
            };

            let display_name = match self.get_text_from_path(&author_info_json, &["displayName"]) {
                Some(key) => key,
                None => {
                    println!("Failed to get display name from index: {}", index);
                    "MISSING_DISPLAY_NAME".to_string()
                }
            };

            let user_verified = match self.get_text_from_path(&author_info_json, &["isVerified"]) {
                Some(key) => string_to_bool(&key).unwrap_or(false),
                None => {
                    println!("Failed to get user verification from index: {}", index);
                    false
                }
            };

            let thumbnail = match self.get_text_from_path(&author_info_json, &["avatarThumbnailUrl"]) {
                Some(key) => key,
                None => {
                    println!("Failed to get avatar thumnail URL for index {}", index);
                    "MISSING AVATAR THUMBNAIL".to_string()
                }
            };

            let comment_id = match self.get_text_from_path(&comment_properties_json, &["commentId"]) {
                Some(comment_id) => comment_id,
                None => {
                    println!("Failed to aquire comment id on index {}", index);
                    "MISSING_COMMENT_ID".to_string()
                }
            };

            let content = match self.get_text_from_path(&comment_properties_json, &["content", "content"]) {
                Some(content) => content,
                None => {
                    println!("Missing content on index {}", index);
                    "MISSING_CONTENT".to_string()
                }
            };

            let published_time = match self.get_text_from_path(&comment_properties_json, &["publishedTime"]) {
                Some(pub_time) => pub_time,
                None => {
                    println!("Missing published time on index {}", index);
                    "MISSING_PUBLISHED_TIME".to_string()
                }
            };

            let toolbar_json = match comment_content
                .get("payload")
                .and_then(|p| p.get("commentEntityPayload"))
                .and_then(|c| c.get("toolbar")) {
                    Some(toolbar_json_section) => toolbar_json_section,
                    None => {
                        println!("The toolbar section for like counts could not be located on index {}", index);
                        &json!({})
                    }
                };

            let comment_info = Comment{
                comment_id,
                channel_id,
                display_name,
                user_verified,
                thumbnail,
                content,
                published_time
            };

            comments.push(comment_info);
        }

        Some(comments)
    }

    pub async fn comments_request(&self, api_key: &String, continuation: &String) -> Result<Value, Box<dyn std::error::Error>> {
        let url = format!("https://www.youtube.com/youtubei/v1/next?key={api_key}");
        let client = reqwest::Client::new();
        
        let decoded_continuation = if continuation.contains('%') {
            let decoded = percent_decode_str(continuation)
                .decode_utf8()
                .unwrap()
                .to_string();
            decoded
        } else {
            continuation.to_string()
        };

        let payload = json!({
        "context": {
            "client": {
                "clientName": "WEB",
                "clientVersion": "2.20240304.00.00"
            }
        },
        "continuation": decoded_continuation
    });
        
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .header("Accept", "application/json")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Origin", "https://www.youtube.com")
            .header("Referer", "https://www.youtube.com/")
            .json(&payload)
            .send()
            .await
            .map_err(|e| YoutubeError::ApiRequestError(Box::new(e)))?;

        println!("Response Status: {}", response.status());

        let response_text = response.text().await?;

        let response_json: Value = serde_json::from_str(&response_text)?;

        let response_string = serde_json::to_string_pretty(&response_json).unwrap_or_default();
        fs::write("comment_response.json", response_string).await.unwrap();

        Ok(response_json)
    }

    pub fn get_api_key(&self, ytcfg: &Value) -> Result<String, YoutubeError> {
        self.get_text_from_path(&ytcfg, &["INNERTUBE_API_KEY"]).ok_or(YoutubeError::ApiKeyNotFound)
    }
    pub fn generate_synthetic_continuation_token(&self, video_id: &str) -> String {
        println!("🥎🥎 Using a synthetic continuation token!! 🥎🥎");
        let token = format!("\x12\r\x12\x0b{video_id}\x18\x062'\"\\x11\"\x0b{video_id}0\x00x\x020\x00B\x10comments-section");
        general_purpose::STANDARD.encode(token.as_bytes())
    }
    pub fn get_continuation_token(&self, data: &Value, video_id: &str) -> String {
        self.get_text_from_path(data, &[
            "engagementPanels", "0", "engagementPanelSectionListRenderer",
            "content", "sectionListRenderer", "contents", "0",
            "itemSectionRenderer", "contents", "0", "continuationItemRenderer",
            "continuationEndpoint", "continuationCommand", "token"
        ]).unwrap_or_else( || {
            self.generate_synthetic_continuation_token(video_id)
        })
    }

    pub async fn get_comments(&self, data: &Value, ytcfg: &Value, video_id: &str) -> Result<String, YoutubeError> {
        let continuation_token = self.get_continuation_token(&data, &video_id);
        let api_key = self.get_api_key(&ytcfg)?;

        println!("Continuation Token: {}", continuation_token);
        println!("API Key: {}", api_key);
        
        let comments_data = self.comments_request(& api_key, &continuation_token).await?;

        let comments = self.comment_extractor(&comments_data).await.unwrap_or_default();

        println!("Captured {} comments!", comments.len());

        self.comment_data_to_json(&comments).await;
        
        Ok("Got comments".to_string())
    }
}
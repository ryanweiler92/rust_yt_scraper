use async_std::fs;
use serde_json::{json, Value};
use base64::{Engine as _, engine::general_purpose};
use reqwest;
use std::collections::HashMap;
use percent_encoding::percent_decode_str;

use crate::models::Comment;
use crate::models::CommentContent;
use helper::string_to_bool;


use super::{helper, YoutubeExtractor};
use super::error_msgs::YoutubeError;

impl YoutubeExtractor {

    pub async fn comment_data_to_json(&self, data: &Vec<Comment>) -> Result<String, Box<dyn std::error::Error>> {
        let string_json = serde_json::to_string_pretty(data)?;
        fs::write("5_final_comment_data_5.json", string_json).await?;
        Ok("Successfully wrote comments to COMMENT.json".to_string())
    }

    pub async fn reply_extractor(&self, api_key: &String, continuation_token: &String, reply_count: &i32, comment_id: &String, video_id: &str) -> Option<Vec<Comment> > {
        let replies_json = self.comments_request(&api_key, &continuation_token).await.unwrap_or_default();
        let replies_json_string = serde_json::to_string_pretty(&replies_json).unwrap_or_default();
        fs::write("4_replies_4.json", replies_json_string).await;

        let replies_usize: usize = *reply_count as usize;

        let mut replies: Vec<Comment> = Vec::with_capacity(replies_usize);

        let reply_content_list_actual = replies_json
            .get("frameworkUpdates")?
            .get( "entityBatchUpdate")?
            .get("mutations")?
            .as_array()?;

        let mut running_reply_count= 0;

        for (index, comment_content) in reply_content_list_actual.iter().enumerate() {
            let reply_content = match self.get_comment_info(comment_content, video_id).await {
                Some(content) => {
                    running_reply_count += 1;
                    content
                },
                None => {
                    // println!("Skipping reply comment at index {} due to missing data", index);
                    continue;
                }
            };
            let reply = Comment::from_comment_content(reply_content, 1, comment_id.clone(), running_reply_count);
            replies.push(reply);
        }
        let extraction_diff = reply_count - replies.len() as i32;
        println!("COMMENT ID: {}, Looking for {} replies. Extracted {} replies. Missing {} replies.",comment_id, reply_count, replies.len(), extraction_diff);

        Some(replies)
    }

    pub async fn get_comment_info(&self, comment_content_json: &Value, video_id: &str) -> Option<CommentContent>{
        let author_info_json = match comment_content_json
            .get("payload")
            .and_then(|p| p.get("commentEntityPayload"))
            .and_then(|c| c.get("author")) {
            Some(author_json) => author_json,
            None => {
                // println!("Could not locate author section.");
                return None
            }
        };

        let comment_properties_json = match comment_content_json
            .get("payload")
            .and_then(|p| p.get("commentEntityPayload"))
            .and_then(|c| c.get("properties")) {
            Some(properties_json) => properties_json,
            None => return None
        };

        let empty_toolbar_json = json!({});
        let toolbar_json = comment_content_json
            .get("payload")
            .and_then(|p| p.get("commentEntityPayload"))
            .and_then(|c| c.get("toolbar"))
            .unwrap_or(&empty_toolbar_json);

        let entity_key = match self.get_text_from_path(&comment_content_json, &["entityKey"]) {
            Some(key) => key,
            None => return None
        };

        let channel_id = self.get_text_from_path(&author_info_json, &["channelId"])
            .unwrap_or_else(|| "MISSING_CHANNEL_ID".to_string());

        let display_name = self.get_text_from_path(&author_info_json, &["displayName"]).unwrap_or_else(|| "MISSING_DISPLAY_NAME".to_string());

        let user_verified = match self.get_text_from_path(&author_info_json, &["isVerified"]) {
            Some(key) => string_to_bool(&key).unwrap_or(false),
            None => {
                false
            }
        };

        let thumbnail = self.get_text_from_path(&author_info_json, &["avatarThumbnailUrl"])
            .unwrap_or_else(|| "MISSING_THUMBNAIL".to_string());

        let comment_id = self.get_text_from_path(&comment_properties_json, &["commentId"])
            .unwrap_or_else(|| "MISSING_COMMENT_ID".to_string());

        let content = self.get_text_from_path(&comment_properties_json, &["content", "content"])
            .unwrap_or_else(|| "MISSING_CONTENT".to_string());

        let published_time = self.get_text_from_path(&comment_properties_json, &["publishedTime"])
            .unwrap_or_else(||"MISSING_PUBLISHED_TIME".to_string());

        let like_count = match self.get_text_from_path(&toolbar_json, &["likeCountNotliked"]) {
            Some(like) => like.parse().unwrap_or_default(),
            None => 0
        };

        let reply_count = match self.get_text_from_path(&toolbar_json, &["replyCount"]) {
            Some(reply) => {
                if reply.is_empty(){
                    0
                } else {
                    reply.parse().unwrap_or_default()
                }
            },
            None => 0
        };

        Some(CommentContent{
            comment_id,
            channel_id,
            video_id: video_id.to_string(),
            display_name,
            user_verified,
            thumbnail,
            content,
            published_time,
            like_count,
            reply_count,
        })
    }
    pub async fn comment_extractor(&self, data: &Value, api_key: &String, video_id: &str) -> Option<Vec<Comment>> {
        let mut comments: Vec<Comment> = Vec::new();

        // Remove each of these lines after debug
        let comment_content_list_test = data
            .get("frameworkUpdates")?
            .get( "entityBatchUpdate")?
            .get("mutations")?;
        let comment_test_string_pretty = serde_json::to_string_pretty(&comment_content_list_test).unwrap_or_default();
        fs::write("2_main_comment_content_2.json", comment_test_string_pretty).await;

        let comment_content_list_actual = data
            .get("frameworkUpdates")?
            .get( "entityBatchUpdate")?
            .get("mutations")?
            .as_array()?;

        println!("comment_content_list_actual length == {}", comment_content_list_actual.len());
        let continuation_items_list_actual = data
            .get("onResponseReceivedEndpoints")?
            .get(1)?
            .get("reloadContinuationItemsCommand")?
            .get("continuationItems")?
            .as_array()?;

        let continuation_list_test = serde_json::to_string_pretty(&continuation_items_list_actual).unwrap_or_default();
        fs::write("3_continuation_items_3.json", continuation_list_test).await;

        for (index, comment_content) in comment_content_list_actual.iter().enumerate() {
            let comment_content = match self.get_comment_info(comment_content, video_id).await {
                Some(content) => content,
                None => {
                    // println!("Skipping comment at index {} due to missing data", index);
                    continue; // Skip this comment and continue with the next one
                }
            };

            if (comment_content.reply_count > 0) {
                let mut comment_continuation_token = "".to_string();

                for continuation_block in continuation_items_list_actual.iter() {
                    let continuation_comment_id = self.get_text_from_path(&continuation_block, &["commentThreadRenderer", "commentViewModel", "commentViewModel", "commentId"]).unwrap_or_default();

                    if &continuation_comment_id == &comment_content.comment_id {
                        comment_continuation_token = self.get_text_from_path(&continuation_block, &["commentThreadRenderer", "replies", "commentRepliesRenderer", "contents", "0", "continuationItemRenderer", "continuationEndpoint", "continuationCommand", "token"]).unwrap_or_default();
                        if !comment_continuation_token.is_empty(){
                            break;
                        }
                    }

                }

                if (comment_continuation_token.is_empty()) {
                    println!("Failed to retrieve continuation token...")
                } else {
                    println!("Continuation Token {}", comment_continuation_token);
                    let mut replies = self.reply_extractor(&api_key, &comment_continuation_token, &comment_content.reply_count, &comment_content.comment_id, &video_id).await?;
                    comments.append(&mut replies);
                }
            } else {
                println!("Did not attempt to get continuation token because replies are 0.")
            }

            let owned_video_id = video_id.to_owned();

            let comment_info = Comment{
                comment_id: comment_content.comment_id,
                channel_id: comment_content.channel_id,
                display_name: comment_content.display_name,
                video_id: owned_video_id,
                user_verified: comment_content.user_verified,
                thumbnail: comment_content.thumbnail,
                content: comment_content.content,
                published_time: comment_content.published_time,
                like_count: comment_content.like_count,
                reply_count: comment_content.reply_count,
                comment_level: 0,
                reply_order: 0,
                reply_to: "".to_string(),

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

        println!("Main Comment Response Status: {}", response.status());

        let response_text = response.text().await?;

        let response_json: Value = serde_json::from_str(&response_text)?;

        let response_string = serde_json::to_string_pretty(&response_json).unwrap_or_default();
        fs::write("1_main_comment_response_1.json", response_string).await.unwrap();

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
        
        let comments_data = self.comments_request(&api_key, &continuation_token).await?;

        let comments = self.comment_extractor(&comments_data, &api_key, video_id).await.unwrap_or_default();

        println!("Captured {} comments!", comments.len());

        self.comment_data_to_json(&comments).await;
        
        Ok("Got comments".to_string())
    }
}
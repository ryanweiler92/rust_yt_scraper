use async_std::fs;
use serde_json::{json, Value};
use base64::{Engine as _, engine::general_purpose};
use reqwest;
use percent_encoding::percent_decode_str;
use tracing::{info, warn, error, debug, trace, instrument};

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

    pub async fn get_next_continuation_token(&self, data: &Value, request_count: &usize) -> Option<String> {
        // Continuation tokens are in different path after initial request
        let not_initial_request = request_count > &1;
        let continuation_item_obj_alias = match not_initial_request {
            true => "appendContinuationItemsAction",
            false => "reloadContinuationItemsCommand"
        };
        let continuation_items_index = match not_initial_request {
            true => 0,
            false => 1
        };

        let continuation_items = data
            .get("onResponseReceivedEndpoints")?
            .get(continuation_items_index)?
            .get(continuation_item_obj_alias)?
            .get("continuationItems")?
            .as_array()?;

        for item in continuation_items.iter().rev() {
            if let Some(token) = self.get_text_from_path(item, &[
                "continuationItemRenderer",
                "continuationEndpoint",
                "continuationCommand",
                "token"
            ]) {
                if !token.is_empty() {
                    return Some(token);
                }
            }
        }
        None
    }

    pub async fn reply_extractor(&self, api_key: &String, continuation_token: &String, reply_count: &i32, comment_id: &String, video_id: &str, create_json_files: bool) -> Option<Vec<Comment> > {
        let replies_json = self.comments_request(&api_key, &continuation_token, &0, false).await.unwrap_or_default();

        if create_json_files{
            let replies_json_string = serde_json::to_string_pretty(&replies_json).unwrap_or_default();
            fs::write("4_replies_4.json", replies_json_string).await;
        }

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
                    continue;
                }
            };
            let reply = Comment::from_comment_content(reply_content, 1, comment_id.clone(), running_reply_count);
            replies.push(reply);
        }
        let extraction_diff = reply_count - replies.len() as i32;
        if extraction_diff > 0 {
            debug!(
                comment_id = %comment_id,
                expected_replies = reply_count,
                extracted_replies = replies.len(),
                missing_replies = extraction_diff,
                "Reply extraction summary"
            );
        }

        Some(replies)
    }

    pub async fn get_comment_info(&self, comment_content_json: &Value, video_id: &str) -> Option<CommentContent>{
        let author_info_json = match comment_content_json
            .get("payload")
            .and_then(|p| p.get("commentEntityPayload"))
            .and_then(|c| c.get("author")) {
            Some(author_json) => author_json,
            None => {
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
    pub async fn comment_extractor(&self, data: &Value, api_key: &String, video_id: &str, request_count: &usize, create_json_files: bool) -> Option<Vec<Comment>> {
        let mut comments: Vec<Comment> = Vec::new();

        if create_json_files{
            let comment_content_list_test = data
                .get("frameworkUpdates")?
                .get( "entityBatchUpdate")?
                .get("mutations")?;
            let comment_test_string_pretty = serde_json::to_string_pretty(&comment_content_list_test).unwrap_or_default();
            let main_comment_file_name = format!("2_{}_main_comment_content_2_{}.json", request_count, request_count);
            fs::write(main_comment_file_name, comment_test_string_pretty).await;
        }

        let comment_content_list_actual = data
            .get("frameworkUpdates")?
            .get( "entityBatchUpdate")?
            .get("mutations")?
            .as_array()?;

        debug!("comment_content_list_actual length == {}", comment_content_list_actual.len());

        let not_initial_request = request_count > &1;
        let continuation_item_obj_alias = match not_initial_request {
            true => "appendContinuationItemsAction",
            false => "reloadContinuationItemsCommand"
        };
        let continuation_items_index = match not_initial_request {
            true => 0,
            false => 1
        };

        let continuation_items_list_actual = data
            .get("onResponseReceivedEndpoints")?
            .get(continuation_items_index)?
            .get(continuation_item_obj_alias)?
            .get("continuationItems")?
            .as_array()?;

        if create_json_files{
            let continuation_list_test = serde_json::to_string_pretty(&continuation_items_list_actual).unwrap_or_default();
            let continuation_file_name = format!("3_{}_continuation_items_3_{}.json", request_count, request_count);
            fs::write(continuation_file_name, continuation_list_test).await;
        }

        for (index, comment_content) in comment_content_list_actual.iter().enumerate() {
            let comment_content = match self.get_comment_info(comment_content, video_id).await {
                Some(content) => content,
                None => {
                    continue;
                }
            };

            // Getting individual reply continuation token
            if comment_content.reply_count > 0 {
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
                    warn!("Failed to retrieve continuation token...")
                } else {
                    let mut replies = self.reply_extractor(&api_key, &comment_continuation_token, &comment_content.reply_count, &comment_content.comment_id, &video_id, create_json_files).await?;
                    comments.append(&mut replies);
                }
            } else {
                debug!("Did not attempt to get continuation token because replies are 0.")
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

    pub async fn comments_request(&self, api_key: &String, continuation: &String, request_count: &usize, create_json_files: bool) -> Result<Value, Box<dyn std::error::Error>> {
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

        debug!("Comment Request Response Status: {}", response.status());

        let response_text = response.text().await?;

        let response_json: Value = serde_json::from_str(&response_text)?;

        if request_count != &0 && create_json_files{
            let response_string = serde_json::to_string_pretty(&response_json).unwrap_or_default();
            let main_comment_file_name = format!("1_{}_main_comment_response_1_{}.json", request_count, request_count);
            fs::write(main_comment_file_name, response_string).await.unwrap();
        }

        Ok(response_json)
    }

    pub fn get_api_key(&self, ytcfg: &Value) -> Result<String, YoutubeError> {
        self.get_text_from_path(&ytcfg, &["INNERTUBE_API_KEY"]).ok_or(YoutubeError::ApiKeyNotFound)
    }
    pub fn generate_synthetic_continuation_token(&self, video_id: &str) -> String {
        warn!("🥎🥎 Using a synthetic continuation token!! 🥎🥎");
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

    pub async fn get_comments(&self, data: &Value, ytcfg: &Value, video_id: &str, max_requests: Option<usize>, create_json_files: bool) -> Result<String, YoutubeError> {
        let initial_continuation_token = self.get_continuation_token(&data, &video_id);
        let api_key = self.get_api_key(&ytcfg)?;

        let mut all_comments: Vec<Comment> = Vec::new();
        let mut current_continuation = initial_continuation_token;
        let mut request_count = 0;
        let max_reqs = max_requests.unwrap_or(50);

        loop {
            request_count += 1;
            debug!("Making request #{} for comments...", request_count);

            let comments_data = match self.comments_request(&api_key, &current_continuation, &request_count, create_json_files).await {
                Ok(data) => data,
                Err(e) => {
                    error!("Error fetching comments on request {}: {:?}", request_count, e);
                    break;
                }
            };

            let batch_comments = self.comment_extractor(&comments_data, &api_key, video_id, &request_count, create_json_files)
                .await
                .unwrap_or_default();

            debug!("Extracted {} comments from batch #{}", batch_comments.len(), request_count);

            all_comments.extend(batch_comments);

            let next_continuation_token = self.get_next_continuation_token(&comments_data, &request_count).await;

            match next_continuation_token {
                Some(token) if !token.is_empty() => {
                    current_continuation = token;
                    debug!("Found next continuation token, continuing...");

                    if request_count >= max_reqs {
                        debug!("Reached maximum request limit ({}), stopping.", max_reqs);
                        break;
                    }

                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
                _ => {
                    debug!("No more continuation tokens found. Finished fetching comments.");
                    break;
                }
            }
        }

        debug!("Total comments captured: {}", all_comments.len());
        debug!("Made {} API requests", request_count);

        if create_json_files {
            self.comment_data_to_json(&all_comments).await;
        }
        self.comment_data_to_json(&all_comments).await;

        Ok(format!("Successfully fetched {} comments in {} requests", all_comments.len(), request_count))
    }
}
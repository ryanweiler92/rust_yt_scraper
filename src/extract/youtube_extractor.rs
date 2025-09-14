use async_std::fs;
use serde_json::{json, Value};
use regex::Regex;
use crate::models::VideoInfo;
use crate::models::Comment;
pub struct YoutubeExtractor;
use tracing::{info, error, debug, instrument};


impl YoutubeExtractor{

    pub fn new() -> Self {
        Self
    }

    #[instrument(skip(self))]
    pub async fn extract(&self, video: &str) -> Result<(VideoInfo, Vec<Comment>), Box<dyn std::error::Error>>  {
        let create_json_files = false;
        let video_id = self.extract_video_id(video).unwrap_or_default();

        info!("Beginning extraction for video ID: {}", video_id);

        let webpage = self.get_json(&video_id).await.unwrap_or_default();
        let initial_data = self.extract_initial_data(&webpage).unwrap_or_default();

        if create_json_files {
            let json_str = serde_json::to_string_pretty(&initial_data).unwrap_or_default();
            fs::write("output.json", json_str).await.unwrap();
        }

        let mut video_info = self.extract_video_info(&initial_data);

        if video_info.yt_id.is_empty() {
            video_info.yt_id = video_id.clone();
        }

        if create_json_files {
            self.save_video_info_to_json(&video_info, "video_info.json").await?;
        }

        debug!(
        title = %video_info.title,
        channel = %video_info.channel,
        channel_id = %video_info.channel_id,
        yt_id = %video_info.yt_id,
        views = video_info.views,
        comment_count = video_info.comment_count,
        like_count = video_info.like_count,
        upload_date = %video_info.upload_date,
        "Extracted video information"
    );

        info!("Extracted video metadata...");

        let ytcfg = self.extract_ytcfg(&webpage, create_json_files).await?;

        let comments = self.get_comments(&initial_data, &ytcfg, &video_id, Some(25), create_json_files).await;



        match comments {
            Ok(comments_data) => {
                debug!(comments_length = comments_data.len(), "Successfully extracted comments");
                info!("Extracted {} comments...", comments_data.len());
                Ok((video_info, comments_data))

            }
            Err(e) => {
                error!(error = %e, video_id = &video_info.yt_id, "Failed to extract comments");
                Ok((video_info, Vec::new()))
            }
        }
    }

    pub async fn extract_ytcfg(&self, webpage: &str, create_json_files: bool) -> Result<Value, Box<dyn std::error::Error>> {
        // Pattern 1: ytcfg.set({...})
        let pattern1 = Regex::new(r"ytcfg\.set\s*\(\s*(\{.+?\})\s*\)")?;
        if let Some(captures) = pattern1.captures(webpage) {
            if let Some(json_str) = captures.get(1) {
                match serde_json::from_str::<Value>(json_str.as_str()) {
                    Ok(data) => {
                        if create_json_files{
                            let json_str = serde_json::to_string_pretty(&data).unwrap_or_default();
                            fs::write("ytcfg_p1.json", &json_str).await.unwrap();
                        }
                        return Ok(data)
                    },
                    Err(_) => {}
                }
            }
        }

        // Pattern 2: window["ytcfg"] = ... ytcfg.set({...})
        let pattern2 = Regex::new(r#"window\["ytcfg"\].*?ytcfg\.set\s*\(\s*(\{.+?\})\s*\)"#)?;
        if let Some(captures) = pattern2.captures(webpage) {
            if let Some(json_str) = captures.get(1) {
                match serde_json::from_str::<Value>(json_str.as_str()) {
                    Ok(data) => {
                        let json_str = serde_json::to_string_pretty(&data).unwrap_or_default();
                        fs::write("ytcfg_p2.json", &json_str).await.unwrap();
                        return Ok(data)
                    },
                    Err(_) => {}
                }
            }
        }
        
        debug!("🥎🥎 Using hardcoded keys. 🥎🥎");
        // Fallback to hardcoded values
        Ok(json!({
        "INNERTUBE_API_KEY": "AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8",
        "INNERTUBE_CONTEXT": {
            "client": {
                "clientName": "WEB",
                "clientVersion": "2.0"
            }
        }
    }))
    }

    fn extract_initial_data(&self, webpage: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let patterns = [
            "window[\"ytInitialData\"] = ",
            "window['ytInitialData'] = ",
            "ytInitialData = ",
            "var ytInitialData = "
        ];

        for pattern in &patterns {
            if let Some(start) = webpage.find(pattern) {
                let json_start = start + pattern.len();

                if let Some(end) = self.find_json_end(&webpage[json_start..]) {
                    let json_str = &webpage[json_start..json_start + end];

                    match serde_json::from_str::<Value>(json_str) {
                        Ok(data) => return Ok(data),
                        Err(_) => continue,
                    }
                }
            }
        }
        Err("Could not extract ytInitialData".into())
    }

    fn find_json_end(&self, text: &str) -> Option<usize> {
        let mut depth = 0;
        let mut in_string = false;
        let mut escaped = false;

        for (i, ch) in text.char_indices() {
            if escaped {
                escaped = false;
                continue;
            }

            match ch {
                '\\' if in_string => escaped = true,
                '"' => in_string = !in_string,
                '{' if !in_string => depth += 1,
                '}' if !in_string => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i + 1);
                    }
                },
                _ => {}
            }
        }
        None
    }

    async fn get_json(&self, video_id: &String) -> Result<String, Box<dyn std::error::Error>>{
        let url = format!("https://www.youtube.com/watch?v={}&bpctr=9999999999&has_verified=1", video_id);
        let html = reqwest::get(&url).await;
        let webpage = html.unwrap().text().await?;
        Ok(webpage)
    }

    fn parse_count(&self, text: &str) -> Option<u64> {
        let text = text.to_lowercase().replace(&[',', ' '][..], "");

        if text.ends_with('k') {
            text.trim_end_matches('k').parse::<f64>().ok()
                .map(|n| (n * 1000.0) as u64)
        } else if text.ends_with('m') {
            text.trim_end_matches('m').parse::<f64>().ok()
                .map(|n| (n * 1000000.0) as u64)
        } else if text.ends_with('b') {
            text.trim_end_matches('b').parse::<f64>().ok()
                .map(|n| (n * 1000000000.0) as u64)
        } else {
            text.parse().ok()
        }
    }

    pub fn get_text_from_path(&self, data: &Value, path: &[&str]) -> Option<String> {
        let mut current = data;

        for key in path {
            current = if key.chars().all(char::is_numeric) {
                let index: usize = key.parse().ok()?;
                current.as_array()?.get(index)?
            } else {
                current.get(key)?
            }
        }

        if let Some(s) = current.as_str() {
            return Some(s.to_string());
        }

        if let Some(b) = current.as_bool() {
            return Some(b.to_string());
        }

        if let Some(runs) = current.get("runs").and_then(|r| r.as_array()) {
            let text: String = runs
                .iter()
                .filter_map(|run| run.get("text")?.as_str())
                .collect();
            if !text.is_empty() {
                return Some(text);
            }
        }
        None
    }

    fn get_views(&self, data: &Value) -> u64{
        let views_string = self.get_text_from_path(&data, &[
            "playerOverlays", "playerOverlayRenderer", "videoDetails", "playerOverlayVideoDetailsRenderer",
            "subtitle", "runs", "2", "text"
        ]).unwrap_or_default();

        views_string
            .split_whitespace()
            .next()
            .and_then(|split| self.parse_count(split))
            .unwrap_or(0)
    }

    fn get_comment_count(&self, data: &Value) -> u64{
        let comment_count_string = self.get_text_from_path(&data, &[
            "engagementPanels", "0", "engagementPanelSectionListRenderer", "header", "engagementPanelTitleHeaderRenderer",
            "contextualInfo", "runs", "0", "text"
        ]).unwrap_or_default();

        self.parse_count(comment_count_string.as_str()).unwrap_or(0)
    }

    fn get_likes(&self, data: &Value) -> u64{
        let like_count_string = self.get_text_from_path(&data, &[
            "contents", "twoColumnWatchNextResults", "results", "results", "contents",
            "0", "videoPrimaryInfoRenderer", "videoActions", "menuRenderer",
            "topLevelButtons", "0", "segmentedLikeDislikeButtonViewModel",
            "likeButtonViewModel", "likeButtonViewModel", "toggleButtonViewModel",
            "toggleButtonViewModel", "defaultButtonViewModel", "buttonViewModel",
            "accessibilityText"
        ]).unwrap_or_default();

        like_count_string
            .split_whitespace()
            .nth(5)
            .and_then(|num| self.parse_count(num))
            .unwrap_or(0)
    }

    fn get_channel_id(&self, data: &Value) -> String {
        self.get_text_from_path(&data, &[
            "contents", "twoColumnWatchNextResults", "results",
            "results", "contents", "1", "videoSecondaryInfoRenderer",
            "subscribeButton", "subscribeButtonRenderer", "channelId"
        ]).unwrap_or_default()
    }
    
    fn get_video_thumbnail(&self, data: &Value) -> String {
        self.get_text_from_path(&data, &[
            "contents", "twoColumnWatchNextResults", "results",
            "results", "contents", "1",
            "videoSecondaryInfoRenderer", "owner", "videoOwnerRenderer", "thumbnail",
            "thumbnails", "0", "url"
        ]).unwrap_or_default()
    }
    
    fn get_upload_date(&self, data: &Value) -> String {
        self.get_text_from_path(&data, &[
            "contents", "twoColumnWatchNextResults", "results",
            "results", "contents", "0", "videoPrimaryInfoRenderer",
            "dateText", "simpleText"
        ]).unwrap_or_default()
    }
    
    fn get_channel_thumbnail(&self, data: &Value) -> String {
        self.get_text_from_path(&data, &[
            "contents", "twoColumnWatchNextResults", "results",
            "results", "contents", "1", "videoSecondaryInfoRenderer",
            "owner", "videoOwnerRenderer", "thumbnail", "thumbnails",
            "2", "url"
            
        ]).unwrap_or_default()
    }

    fn extract_video_info(&self, initial_data: &Value) -> VideoInfo {
        VideoInfo {
            title: self.get_text_from_path(initial_data, &[
                "contents", "twoColumnWatchNextResults", "results",
                "results", "contents", "0", "videoPrimaryInfoRenderer", "title"
            ]).unwrap_or_default(),

            channel: self.get_text_from_path(initial_data, &[
                "contents", "twoColumnWatchNextResults", "results",
                "results", "contents", "1", "videoSecondaryInfoRenderer",
                "owner", "videoOwnerRenderer", "title"
            ]).unwrap_or_default(),

            channel_id: self.get_channel_id(initial_data),

            description: self.get_text_from_path(initial_data, &[
                "contents", "twoColumnWatchNextResults", "results", "results", "contents",
                "1", "videoSecondaryInfoRenderer", "attributedDescription", "content"
            ]).unwrap_or_default(),

            yt_id: self.get_text_from_path(initial_data, &[
                "contents", "twoColumnWatchNextResults", "secondaryResults", "secondaryResults",
                "results", "0", "compactVideoRenderer", "videoId"
            ]).unwrap_or_default(),

            views: self.get_views(initial_data),

            comment_count: self.get_comment_count(initial_data),

            like_count: self.get_likes(initial_data),
            
            video_thumbnail: self.get_video_thumbnail(initial_data),
            
            upload_date: self.get_upload_date(initial_data),
            
            channel_thumbnail: self.get_channel_thumbnail(initial_data)
        }
    }

    fn extract_video_id(&self, input: &str) -> Option<String> {
        if input.len() == 11 && !input.contains('/') {
            Some(input.to_string())
        } else {
            input.split("v=")
                .nth(1)?
                .split('&')
                .next()
                .map(|s| s.to_string())
        }
    }

    pub async fn save_video_info_to_json(&self, video_info: &VideoInfo, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json_str = serde_json::to_string_pretty(video_info)?;
        fs::write(file_path, json_str).await?;
        info!("Video info saved to {}", file_path);
        Ok(())
    }
}
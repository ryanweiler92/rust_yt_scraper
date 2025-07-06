use async_std::fs;
use serde_json::{json, Value};
use regex::Regex;

use crate::models::VideoInfo;

pub struct YoutubeExtractor;

impl YoutubeExtractor{

    pub fn new() -> Self {
        Self
    }

    pub async fn extract_ytcfg(&self, webpage: &str) -> Result<Value, Box<dyn std::error::Error>> {
        // Pattern 1: ytcfg.set({...})
        let pattern1 = Regex::new(r"ytcfg\.set\s*\(\s*(\{.+?\})\s*\)")?;
        if let Some(captures) = pattern1.captures(webpage) {
            if let Some(json_str) = captures.get(1) {
                match serde_json::from_str::<Value>(json_str.as_str()) {
                    Ok(data) => {
                        let json_str = serde_json::to_string_pretty(&data).unwrap_or_default();
                        fs::write("ytcfg_p1.json", &json_str).await.unwrap();
                        return Ok(data)
                    },
                    Err(_) => {} // Continue to next pattern
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
        
        println!("🥎🥎 Using hardcoded values. 🥎🥎");
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

    async fn get_json(&self, video_id: &str) -> Result<String, Box<dyn std::error::Error>>{
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
            "contents", "twoColumnWatchNextResults", "secondaryResults", 
            "secondaryResults", "results", "0", "compactVideoRenderer",
            "thumbnail", "thumbnails", "1", "url"
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

    pub async fn extract(&self, video_id: &str){
        let webpage = self.get_json(video_id).await.unwrap_or_default();
        let initial_data = self.extract_initial_data(&webpage).unwrap_or_default();
        let json_str = serde_json::to_string_pretty(&initial_data).unwrap_or_default();
        fs::write("output.json", json_str).await.unwrap();

        let video_info = self.extract_video_info(&initial_data);
        println!("Title: {}", video_info.title);
        println!("Channel: {}", video_info.channel);
        println!("Channel ID: {}", video_info.channel_id);
        // println!("Description: {}", video_info.description);
        println!("YT ID: {}", video_info.yt_id);
        println!("Views: {}", video_info.views);
        println!("Comment Count: {}", video_info.comment_count);
        println!("Like Count: {}", video_info.like_count);
        println!("Video Thumbnail: {}", video_info.video_thumbnail);
        println!("Upload Date: {}", video_info.upload_date);
        println!("Channel Thumbnail: {}", video_info.channel_thumbnail);
        
        let ytcfg = self.extract_ytcfg(&webpage).await.unwrap();
        
        let comments = self.get_comments(&initial_data, &ytcfg, &video_id).await;
        
        match comments {
            Ok(comments_data) => {
                println!("Comments Data: {}", comments_data)
            }
            Err(e) => {
                println!("There was an error extracting comments: {}", e);
            }
        }
    }
}
use async_std::fs;
use regex::Regex;
use serde_json::Value;

use crate::models::VideoInfo;

pub struct YoutubeExtractor;

impl YoutubeExtractor{

    pub fn new() -> Self {
        Self
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

    fn get_text_from_path(&self, data: &Value, path: &[&str]) -> Option<String> {
        let mut current = data;

        for key in path {
            current = current.get(key)?;
        }

        if let Some(simple_text) = current.get("simpleText") {
            return simple_text.as_str().map(String::from);
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

// /contents/twoColumnWatchNextResults/results/results/contents/0/videoPrimaryInfoRenderer/title/runs/0/text
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
    }
}
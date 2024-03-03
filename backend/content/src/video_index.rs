use std::{
    collections::HashMap,
    fs::{DirEntry, File},
    io::Read,
    path::PathBuf,
};

use regex::Regex;
use tracing::info;

#[derive(Debug)]
pub struct VideoIndex {
    entries: HashMap<u32, VideoIndexEntry>,
}

#[derive(Debug)]
pub struct VideoIndexEntry {
    webvtt: subtp::vtt::WebVtt,
}

impl VideoIndex {
    pub fn empty_index() -> VideoIndex {
        VideoIndex {
            entries: HashMap::new(),
        }
    }

    pub fn from_content_area(
        base_path: &PathBuf,
    ) -> Result<VideoIndex, Box<dyn std::error::Error>> {
        info!("Building index of video content in {:?} ... ", base_path);
        let mut entries: HashMap<u32, VideoIndexEntry> = HashMap::new();
        let mut video_content_count = 0;
        let dir_entries: Result<Vec<DirEntry>, _> = std::fs::read_dir(base_path)?.collect();
        let file_regex = Regex::new(r"fosdem-2024-(?<event_id>\d+)-")?;
        for entry in dir_entries? {
            let file_name = entry.file_name();
            if let Some(c) = file_regex.captures(file_name.to_str().unwrap()) {
                if let Some(m) = c.name("event_id") {
                    let event_id: u32 = m.as_str().parse().unwrap();
                    let mut file = File::open(entry.path())?;
                    let mut content = String::new();
                    file.read_to_string(&mut content)?;
                    let webvtt = subtp::vtt::WebVtt::parse(&content)?;
                    entries.insert(event_id, VideoIndexEntry { webvtt });
                    video_content_count += 1;
                }
            }
        }
        info!("Read {} events with video content ", video_content_count);
        Ok(VideoIndex { entries })
    }

    pub fn webvtt_for_event_id(&self, event_id: u32) -> Option<subtp::vtt::WebVtt> {
        if let Some(entry) = self.entries.get(&event_id) {
            Some(entry.webvtt.clone())
        } else {
            None
        }
    }
}

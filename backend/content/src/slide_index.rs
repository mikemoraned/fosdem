use std::{collections::HashMap, fs::File, io::Read, path::Path};

use shared::model::Event;
use tracing::trace;

#[derive(Debug)]
pub struct SlideIndex {
    pub entries: HashMap<u32, String>,
}

impl SlideIndex {
    pub fn empty_index() -> SlideIndex {
        SlideIndex {
            entries: HashMap::default(),
        }
    }

    pub fn from_content_area(
        base_path: &Path,
        events: &[Event],
    ) -> Result<SlideIndex, Box<dyn std::error::Error>> {
        let mut entries: HashMap<u32, String> = HashMap::new();

        trace!("Fetching slide content from {:?} ... ", base_path);
        let mut slide_content_count = 0;
        for event in events.iter() {
            let slide_content_path = base_path.join(event.id.to_string()).with_extension("txt");
            if slide_content_path.exists() {
                let mut file = File::open(slide_content_path)?;
                let mut slide_content = String::new();
                file.read_to_string(&mut slide_content)?;
                entries.insert(event.id, slide_content);
                slide_content_count += 1;
            }
        }
        trace!("Read {} events with slide content ", slide_content_count);

        Ok(SlideIndex { entries })
    }
}

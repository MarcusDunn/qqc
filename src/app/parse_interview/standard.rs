use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};

use crate::app::{Interview, Section};

#[derive(serde::Deserialize)]
pub struct Root {
    segments: Vec<Segment>,
    speakers: Vec<Speaker>,
}

#[derive(serde::Deserialize)]
struct Speaker {
    #[serde(rename = "spkid")]
    speaker_id: String,
    name: String,
}

#[derive(serde::Deserialize)]
struct Segment {
    #[serde(rename = "speaker")]
    speaker_id: String,
    words: Vec<Word>,
}

#[derive(serde::Deserialize)]
struct Word {
    text: String,
}

impl From<Root> for Interview {
    fn from(root: Root) -> Self {
        Interview {
            speakers: root
                .speakers
                .into_iter()
                .map(|Speaker { speaker_id, name }| {
                    let mut hasher = DefaultHasher::new();
                    speaker_id.hash(&mut hasher);
                    (hasher.finish(), name)
                })
                .collect(),
            sections: root
                .segments
                .into_iter()
                .map(|Segment { speaker_id, words }| Section {
                    speaker_id: {
                        let mut hasher = DefaultHasher::new();
                        speaker_id.hash(&mut hasher);
                        hasher.finish()
                    },
                    text: words.into_iter().map(|Word { text }| text + " ").collect(),
                    codes: BTreeSet::default(),
                })
                .collect(),
        }
    }
}

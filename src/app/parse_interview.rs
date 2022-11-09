use crate::app::Interview;
use tracing::warn;

mod standard;

#[derive(Debug)]
enum InterviewFormat {
    /// not sure what to call this format for now
    Standard(standard::Root),
    Vtt(vtt::Vtt),
}

pub fn file_extensions() -> &'static [&'static str] {
    &["json", "vtt"]
}

impl TryFrom<&str> for InterviewFormat {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        serde_json::from_str::<standard::Root>(value)
            .map_err(|err| {
                warn!(?err, "failed to parse as standard json");
            })
            .map(InterviewFormat::Standard)
            .or_else(|_| {
                vtt::Vtt::try_from(value)
                    .map_err(|err| {
                        warn!(?err, "failed to parse as vtt");
                    })
                    .map(InterviewFormat::Vtt)
            })
    }
}

impl From<InterviewFormat> for Interview {
    fn from(interview_format: InterviewFormat) -> Self {
        match interview_format {
            InterviewFormat::Standard(standard) => standard.into(),
            InterviewFormat::Vtt(vtt) => vtt.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_standard_json() {
        let json = r#"{
    "speakers": [
        {
            "spkid": "spk1",
            "name": "Speaker 1"
        },
        {
            "spkid": "spk2",
            "name": "Speaker 2"
        }
    ],
    "segments": [
        {
            "speaker": "spk1",
            "words": [
                {
                    "start": 3.06,
                    "end": 3.36,
                    "duration": 0.29999995,
                    "text": "Okay,",
                    "conf": 1,
                    "pristine": true
                }
            ]
        }
    ]
}"#;

        InterviewFormat::try_from(json).unwrap();
    }

    #[test]
    fn parse_vtt() {
        let entries = "1
00:00:09.640 --> 00:00:13.459
Marcus Dunn: ewubfqofbweqpfboifjwnpfiwjn pviwljan pilsn pajn wpailfjn aps

2
00:00:13.470 --> 00:00:43.370
Edward Cunningham: ewubfqofbweqpfboifjwnpfiwjn pviwljan pilsn pajn wpailfjn aonwe fi

3
00:00:43.380 --> 00:00:50.870
Marcus Dunn: ewubfqofbweqpfboifjwnpfiwjn pviwljan pilsn pajn wpailfjn owefba ou
";

        InterviewFormat::try_from(entries).unwrap();
    }
}

mod vtt {
    use crate::app::{Interview, Section};
    use std::collections::btree_map::Entry;
    use std::collections::{BTreeMap, BTreeSet};
    use std::num::ParseIntError;
    use std::ops::Deref;
    use tracing::info;

    #[derive(Eq, PartialEq, Debug)]
    pub(crate) struct Vtt(Vec<VttEntry>);

    impl From<Vtt> for Interview {
        fn from(Vtt(vtt): Vtt) -> Self {
            const UNKNOWN_SPEAKER_ID: u64 = 0;

            let mut sections = vec![];
            let mut speakers = BTreeMap::from([(String::from("Unknown"), UNKNOWN_SPEAKER_ID)]);
            let mut speaker_id = UNKNOWN_SPEAKER_ID;
            for VttEntry {
                index: _,
                speaker,
                text,
            } in vtt
            {
                let speaker_id = if let Some(speaker) = speaker {
                    match speakers.entry(speaker) {
                        Entry::Vacant(v) => {
                            speaker_id += 1;
                            *v.insert(speaker_id)
                        }
                        Entry::Occupied(o) => *o.get(),
                    }
                } else {
                    UNKNOWN_SPEAKER_ID
                };
                sections.push(Section {
                    speaker_id,
                    text,
                    codes: BTreeSet::new(),
                });
            }
            Interview {
                speakers: speakers.into_iter().map(|(s, i)| (i, s)).collect(),
                sections,
            }
        }
    }

    impl TryFrom<&str> for Vtt {
        type Error = VttEntryParseError;

        fn try_from(value: &str) -> Result<Self, Self::Error> {
            let normalized = value.replace("\r\n", "\n");
            let split = normalized
                .split("\n\n")
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>();
            info!(?split);
            let skip = split.into_iter().skip(1).collect::<Vec<_>>();
            info!(?skip);
            let map = skip
                .into_iter()
                .map(VttEntry::try_from)
                .collect::<Result<Vtt, _>>();
            info!(?map);
            map
        }
    }

    impl FromIterator<VttEntry> for Vtt {
        fn from_iter<T: IntoIterator<Item = VttEntry>>(iter: T) -> Self {
            Vtt(Vec::from_iter(iter))
        }
    }

    impl Deref for Vtt {
        type Target = Vec<VttEntry>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[derive(Eq, PartialEq, Debug)]
    pub struct VttEntry {
        index: usize,
        speaker: Option<String>,
        text: String,
    }

    #[derive(Debug, Eq, PartialEq)]
    pub enum VttEntryParseError {
        MissingIndex(String),
        MissingTimestamps(String),
        MissingText(String),
        IndexParseError(ParseIntError),
    }

    impl From<ParseIntError> for VttEntryParseError {
        fn from(parse_int_error: ParseIntError) -> Self {
            VttEntryParseError::IndexParseError(parse_int_error)
        }
    }

    impl TryFrom<&str> for VttEntry {
        type Error = VttEntryParseError;

        fn try_from(value: &str) -> Result<Self, Self::Error> {
            let mut lines = value.lines();
            let index = lines
                .next()
                .ok_or_else(|| VttEntryParseError::MissingIndex(value.to_string()))?
                .parse::<usize>()?;
            let _timestamps = lines
                .next()
                .ok_or_else(|| VttEntryParseError::MissingTimestamps(value.to_string()))?;
            let text = lines
                .next()
                .ok_or_else(|| VttEntryParseError::MissingText(value.to_string()))?;
            let (speaker, text) = match text.split_once(": ") {
                None => (None, text),
                Some((speaker, text)) => (Some(speaker), text),
            };
            debug_assert!(lines.next().is_none(), "entry contained more than 3 lines");
            Ok(VttEntry {
                index,
                speaker: speaker.map(String::from),
                text: text.to_string(),
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_parse_single_vtt_entry() {
            let entry = "1
00:00:09.640 --> 00:00:13.459
Marcus Dunn: Yo yo yo yo, this is a test";

            assert_eq!(
                entry.try_into(),
                Ok(VttEntry {
                    index: 1,
                    speaker: "Marcus Dunn".to_string(),
                    text: "Yo yo yo yo, this is a test".to_string(),
                })
            );
        }

        #[test]
        fn test_parse_multiple_vtt_entry() {
            let entries = "WEBVTT

1
00:00:09.640 --> 00:00:13.459
Marcus Dunn: ewubfqofbweqpfboifjwnpfiwjn pviwljan pilsn pajn wpailfjn aps

2
00:00:13.470 --> 00:00:43.370
Edward Cunningham: ewubfqofbweqpfboifjwnpfiwjn pviwljan pilsn pajn wpailfjn aonwe fi

3
00:00:43.380 --> 00:00:50.870
ewubfqofbweqpfboifjwnpfiwjn pviwljan pilsn pajn wpailfjn owefba ou
";
            assert_eq!(
                entries.try_into(),
                Ok(Vtt(vec![
                    VttEntry {
                        index: 1,
                        speaker: Some("Marcus Dunn".to_string()),
                        text: "ewubfqofbweqpfboifjwnpfiwjn pviwljan pilsn pajn wpailfjn aps"
                            .to_string(),
                    },
                    VttEntry {
                        index: 2,
                        speaker: Some("Edward Cunningham".to_string()),
                        text: "ewubfqofbweqpfboifjwnpfiwjn pviwljan pilsn pajn wpailfjn aonwe fi"
                            .to_string(),
                    },
                    VttEntry {
                        index: 3,
                        speaker: None,
                        text: "ewubfqofbweqpfboifjwnpfiwjn pviwljan pilsn pajn wpailfjn owefba ou"
                            .to_string(),
                    },
                ]))
            );
        }
    }
}

pub(crate) fn parse(str: &str) -> Result<Interview, ()> {
    InterviewFormat::try_from(str).map(Interview::from)
}

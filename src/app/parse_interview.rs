use crate::app::Interview;

mod standard;

pub fn from_json_slice(slice: &[u8]) -> serde_json::Result<Interview> {
    serde_json::from_slice::<InterviewFormat>(slice).map(Interview::from)
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
enum InterviewFormat {
    /// not sure what to call this format for now
    Standard(standard::Root),
}

impl From<InterviewFormat> for Interview {
    fn from(interview_format: InterviewFormat) -> Self {
        match interview_format {
            InterviewFormat::Standard(standard) => standard.into(),
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

        from_json_slice(json.as_bytes()).unwrap();
    }
}

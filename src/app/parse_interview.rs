use crate::app::Interview;
use tracing::warn;

mod standard;
mod vtt;

#[derive(Debug)]
enum InterviewFormat {
    /// not sure what to call this format for now
    Standard(standard::Root),
    Vtt(vtt::Vtt),
}

pub fn file_extensions() -> &'static [&'static str] {
    &["json", "vtt", "srt"]
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

pub(crate) fn parse(str: &str) -> Result<Interview, ()> {
    InterviewFormat::try_from(str).map(Interview::from)
}

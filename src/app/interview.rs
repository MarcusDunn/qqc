use std::cmp::min;

use crate::app::{Interview, Section};

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct InterviewSwiper {
    pub interview: Interview,
    index: usize,
}

impl InterviewSwiper {
    pub(crate) fn try_prev(&mut self) -> Option<usize> {
        let option = self.index.checked_sub(1)?;
        if self.interview.sections.get(option).is_some() {
            self.index = option;
            Some(self.index)
        } else {
            None
        }
    }
}

impl InterviewSwiper {
    pub(crate) fn try_next(&mut self) -> Option<usize> {
        let option = self.index.checked_add(1)?;
        if self.interview.sections.get(option).is_some() {
            self.index = option;
            Some(self.index)
        } else {
            None
        }
    }
}

impl InterviewSwiper {
    pub fn new(interview: Interview) -> Self {
        if interview.sections.is_empty() {
            panic!("interview must have at least one section")
        }
        Self {
            interview,
            index: 0,
        }
    }

    pub fn window_mut(
        &mut self,
        behind: usize,
        ahead: usize,
    ) -> (&mut [Section], &mut Section, &mut [Section]) {
        let (before, after) = self.interview.sections.split_at_mut(self.index);
        let before = &mut before[self.index.saturating_sub(behind)..];
        debug_assert!(
            before.len() <= behind,
            "expected {} <= {}",
            before.len(),
            behind
        );
        if let Some((curr, after)) = after.split_first_mut() {
            let after = &mut after[0..min(after.len(), ahead)];
            debug_assert!(
                after.len() <= ahead,
                "expected {} <= {}",
                after.len(),
                ahead
            );
            (before, curr, after)
        } else {
            panic!("index was invalid")
        }
    }

    pub fn current_mut(&mut self) -> &mut Section {
        &mut self.interview.sections[self.index]
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn test_window() {
        let swiper = InterviewSwiper {
            interview: Interview {
                speakers: BTreeMap::default(),
                sections: vec![
                    Section {
                        speaker_id: 0,
                        text: "0th".to_string(),
                        codes: Default::default(),
                    },
                    Section {
                        speaker_id: 0,
                        text: "1st".to_string(),
                        codes: Default::default(),
                    },
                    Section {
                        speaker_id: 0,
                        text: "2nd".to_string(),
                        codes: Default::default(),
                    },
                    Section {
                        speaker_id: 0,
                        text: "3rd".to_string(),
                        codes: Default::default(),
                    },
                    Section {
                        speaker_id: 0,
                        text: "4th".to_string(),
                        codes: Default::default(),
                    },
                ],
            },
            index: 2,
        };
        let (before, curr, after) = swiper.window(1, 1);

        assert_eq!(before.len(), 1);
        assert_eq!(after.len(), 1);
        assert_ne!(before[0].text, curr.text);
        assert_ne!(after[0].text, curr.text);
    }

    #[test]
    fn test_window_2() {
        let swiper = InterviewSwiper {
            interview: Interview {
                speakers: BTreeMap::default(),
                sections: vec![
                    Section {
                        speaker_id: 0,
                        text: "0th".to_string(),
                        codes: Default::default(),
                    },
                    Section {
                        speaker_id: 0,
                        text: "1st".to_string(),
                        codes: Default::default(),
                    },
                    Section {
                        speaker_id: 0,
                        text: "2nd".to_string(),
                        codes: Default::default(),
                    },
                    Section {
                        speaker_id: 0,
                        text: "3rd".to_string(),
                        codes: Default::default(),
                    },
                    Section {
                        speaker_id: 0,
                        text: "4th".to_string(),
                        codes: Default::default(),
                    },
                ],
            },
            index: 0,
        };
        let (before, curr, after) = swiper.window(1, 1);

        assert_eq!(before.len(), 0);
        assert_eq!(after.len(), 1);
        assert_ne!(after[0].text, curr.text);
    }
}

#[test]
fn test_window() {
    let swiper = InterviewSwiper {
        interview: Interview {
            speakers: std::collections::BTreeMap::default(),
            sections: vec![
                Section {
                    speaker_id: 0,
                    text: "0th".to_string(),
                    codes: Default::default(),
                },
                Section {
                    speaker_id: 0,
                    text: "1st".to_string(),
                    codes: Default::default(),
                },
                Section {
                    speaker_id: 0,
                    text: "2nd".to_string(),
                    codes: Default::default(),
                },
                Section {
                    speaker_id: 0,
                    text: "3rd".to_string(),
                    codes: Default::default(),
                },
                Section {
                    speaker_id: 0,
                    text: "4th".to_string(),
                    codes: Default::default(),
                },
            ],
        },
        index: 4,
    };
    let (before, curr, after) = swiper.window(1, 1);

    assert_eq!(before.len(), 1);
    assert_eq!(after.len(), 0);
    assert_ne!(before[0].text, curr.text);
}

#[test]
fn test_window_3() {
    let swiper = InterviewSwiper {
        interview: Interview {
            speakers: std::collections::BTreeMap::default(),
            sections: vec![Section {
                speaker_id: 0,
                text: "0th".to_string(),
                codes: Default::default(),
            }],
        },
        index: 0,
    };
    let (_, curr, _) = swiper.window(0, 0);
    assert_eq!(curr.text, "0th")
}

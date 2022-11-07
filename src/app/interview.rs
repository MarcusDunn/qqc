use crate::app::{Interview, Section};

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct InterviewSwiper {
    pub interview: Interview,
    pub(crate) index: usize,
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

    pub fn window_mut<T>(
        slice: &mut [T],
        curr: usize,
        behind: usize,
        ahead: usize,
    ) -> (&mut [T], &mut T, &mut [T]) {
        let (before, after) = slice.split_at_mut(curr);
        let before = &mut before[curr.saturating_sub(behind)..];
        debug_assert!(
            before.len() <= behind,
            "expected {} <= {}",
            before.len(),
            behind
        );
        if let Some((center, after)) = after.split_first_mut() {
            let len = after.len();
            let after = &mut after[..ahead.min(len.saturating_sub(1))];
            debug_assert!(
                after.len() <= ahead,
                "expected {} <= {}",
                after.len(),
                ahead
            );
            (before, center, after)
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
        let mut swiper = InterviewSwiper {
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
        let (before, curr, after) = InterviewSwiper::window_mut(&mut swiper.interview.sections, 2, 1, 1);

        assert_eq!(before.len(), 1);
        assert_eq!(after.len(), 1);
        assert_ne!(before[0].text, curr.text);
        assert_ne!(after[0].text, curr.text);
    }

    #[test]
    fn test_window_2() {
        let mut swiper = InterviewSwiper {
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
        let (before, curr, after) = InterviewSwiper::window_mut(&mut swiper.interview.sections, 0 , 1, 1);

        assert_eq!(before.len(), 0);
        assert_eq!(after.len(), 1);
        assert_ne!(after[0].text, curr.text);
    }
}

#[test]
fn test_window() {
    let mut swiper = InterviewSwiper {
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
    let (before, curr, after) = InterviewSwiper::window_mut(&mut swiper.interview.sections, 4 , 1, 1);

    assert_eq!(before.len(), 1);
    assert_eq!(after.len(), 0);
    assert_ne!(before[0].text, curr.text);
}

#[test]
fn test_window_3() {
    let mut swiper = InterviewSwiper {
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
    let (before, curr, after) = InterviewSwiper::window_mut(&mut swiper.interview.sections, 0 , 0, 0);
    assert_eq!(before.len(), 0);
    assert_eq!(after.len(), 0);
    assert_eq!(curr.text, "0th")
}

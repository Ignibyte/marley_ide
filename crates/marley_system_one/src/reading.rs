//! What a use may act on.
//!
//! Each answer is read against its question's threshold, and a reading says what came of the
//! call: code's own verdicts, the model's readings, or why there are none. The thresholds are the
//! starting points the design note sets until a use has labels to fit its own: a choice or a score
//! counts at a confidence of 0.5 or more and never for an abstaining option, and a noul between
//! 0.35 and 0.65 is no signal.

use crate::request::{Answer, Answers};
use crate::{Question, QuestionSet};

/// The confidence a choice or a score needs to count.
pub const CONFIDENCE_FLOOR: f64 = 0.5;

/// The lower edge of a noul's no-signal band, widened by 1e-9 so an answer on the edge reads as
/// none.
pub const NOUL_LOW: f64 = 0.35 - 1e-9;

/// The upper edge of a noul's no-signal band, widened by 1e-9.
pub const NOUL_HIGH: f64 = 0.65 + 1e-9;

/// Options that say the state does not settle the question.
pub const ABSTAINING: [&str; 2] = ["cannot_tell", "none"];

/// What came of a use's call.
#[derive(Debug, Clone, PartialEq)]
pub enum Reading {
    /// The layer or the use is off, so nothing was asked.
    Off,
    /// Code's own verdicts, from the `rules` provider.
    Rules(Vec<Read>),
    /// The model's answers, each read against its threshold.
    Model(Vec<Read>),
    /// Refused before any request, and why.
    Refused(String),
    /// No answer came, and why.
    Unavailable(String),
}

/// One question's reading.
#[derive(Debug, Clone, PartialEq)]
pub struct Read {
    /// The question's key.
    pub key: String,
    /// What its answer says.
    pub signal: Signal,
}

/// What one answer says past its threshold, or that it says nothing.
#[derive(Debug, Clone, PartialEq)]
pub enum Signal {
    /// A noul outside its band.
    Noul {
        /// Whether the answer is yes.
        holds: bool,
        /// The probability of yes.
        probability: f64,
    },
    /// A choice at or above the floor.
    Choice {
        /// The option chosen.
        option: String,
        /// The model's confidence.
        confidence: f64,
    },
    /// A score at or above the floor.
    Score {
        /// The expected level.
        score: f64,
        /// The model's confidence.
        confidence: f64,
    },
    /// No signal, and why: inside the band, under the floor, an abstaining option, or no answer.
    Nothing(String),
}

/// Each question of `set`, read from `answers`.
#[must_use]
pub fn read(set: &QuestionSet, answers: &Answers) -> Vec<Read> {
    set.questions
        .iter()
        .map(|question| Read {
            key: question.key().to_string(),
            signal: signal(question, answers.by_key.get(question.key())),
        })
        .collect()
}

/// What `answer` says of `question`.
fn signal(question: &Question, answer: Option<&Answer>) -> Signal {
    match (question, answer) {
        (_, None) => Signal::Nothing("no answer".to_string()),
        (Question::Noul { .. }, Some(Answer::Noul { noul })) => {
            let noul = *noul;
            if !noul.is_finite() {
                Signal::Nothing("the answer is not a number".to_string())
            } else if (NOUL_LOW..=NOUL_HIGH).contains(&noul) {
                Signal::Nothing(format!("{noul:.2} is between 0.35 and 0.65"))
            } else {
                Signal::Noul {
                    holds: noul > NOUL_HIGH,
                    probability: noul,
                }
            }
        }
        (
            Question::Choice { .. },
            Some(Answer::Choice {
                choice, confidence, ..
            }),
        ) => {
            if ABSTAINING.contains(&choice.as_str()) {
                Signal::Nothing(format!("the model chose `{choice}`"))
            } else if *confidence < CONFIDENCE_FLOOR {
                Signal::Nothing(format!("confidence {confidence:.2} is under 0.5"))
            } else {
                Signal::Choice {
                    option: choice.clone(),
                    confidence: *confidence,
                }
            }
        }
        (
            Question::Score { .. },
            Some(Answer::Score {
                score, confidence, ..
            }),
        ) => {
            if *confidence < CONFIDENCE_FLOOR {
                Signal::Nothing(format!("confidence {confidence:.2} is under 0.5"))
            } else {
                Signal::Score {
                    score: *score,
                    confidence: *confidence,
                }
            }
        }
        _ => Signal::Nothing("the answer is of another kind than the question".to_string()),
    }
}

impl Reading {
    /// The reading as one line, for a toast and the System One calls view: `command failed: yes (0.92)`,
    /// `Refused: project not listed`.
    #[must_use]
    pub fn summary(&self) -> String {
        match self {
            Self::Off => "off".to_string(),
            Self::Rules(reads) => format!("{} (rules)", reads_summary(reads, false)),
            Self::Model(reads) => reads_summary(reads, true),
            Self::Refused(reason) => format!("Refused: {reason}"),
            Self::Unavailable(reason) => format!("Unavailable: {reason}"),
        }
    }

    /// Whether the reading is a refusal or no answer, which the System One calls view marks.
    #[must_use]
    pub const fn failed(&self) -> bool {
        matches!(self, Self::Refused(_) | Self::Unavailable(_))
    }
}

/// The reads as one line; `numbers` adds each probability or confidence.
fn reads_summary(reads: &[Read], numbers: bool) -> String {
    let parts: Vec<String> = reads
        .iter()
        .map(|read| {
            let words = read.key.replace('_', " ");
            match &read.signal {
                Signal::Noul { holds, probability } => {
                    let said = if *holds { "yes" } else { "no" };
                    if numbers {
                        format!("{words}: {said} ({probability:.2})")
                    } else {
                        format!("{words}: {said}")
                    }
                }
                Signal::Choice { option, confidence } => {
                    if numbers {
                        format!("{words}: {option} {confidence:.2}")
                    } else {
                        format!("{words}: {option}")
                    }
                }
                Signal::Score { score, confidence } => {
                    if numbers {
                        format!("{words}: {score:.2} ({confidence:.2})")
                    } else {
                        format!("{words}: {score:.2}")
                    }
                }
                Signal::Nothing(reason) => format!("{words}: no signal ({reason})"),
            }
        })
        .collect();
    if parts.is_empty() {
        "no questions".to_string()
    } else {
        parts.join("; ")
    }
}

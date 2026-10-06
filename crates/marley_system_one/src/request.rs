//! The `/v1/systemone` request and its answer, and the same through Cloudflare Workers AI (#548).
//!
//! The request is the published `/v1/systemone` shape; Cloudflare's carries the same `state` and
//! `questions` inside `input` and answers inside its REST envelope's `result`. The answer is read as Jev gives it: a noul's
//! `noul`, a choice's `choice`, `confidence` and `probabilities`, and a score's fractional
//! `score` with its `confidence`, `probabilities` and `legend`, then `model` and `usage`. What the
//! reader does not know it ignores, so a field the provider adds breaks nothing.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::state::cut;
use crate::{Question, QuestionSet};

/// The most characters of an error's body that are kept: a body can echo the request.
pub const ERROR_LIMIT: usize = 300;

/// Jev's name on Cloudflare Workers AI, which carries no version.
pub const CLOUDFLARE_MODEL: &str = "typesafe/jev";

/// The request for `set` about `state`, asked of `model`.
#[must_use]
pub fn build(model: &str, state: &str, set: &QuestionSet) -> Value {
    json!({ "model": model, "state": state, "questions": questions_json(set) })
}

/// The Workers AI request for `set` about `state`: Jev's own `state` and `questions` inside
/// `input`, the model named as Cloudflare names it.
#[must_use]
pub fn build_cloudflare(state: &str, set: &QuestionSet) -> Value {
    json!({
        "model": CLOUDFLARE_MODEL,
        "input": { "state": state, "questions": questions_json(set) },
    })
}

fn questions_json(set: &QuestionSet) -> Map<String, Value> {
    set.questions
        .iter()
        .map(|question| (question.key().to_string(), question_json(question)))
        .collect()
}

/// One question as the request asks it.
fn question_json(question: &Question) -> Value {
    match question {
        Question::Noul {
            instructions,
            when_true,
            when_false,
            ..
        } => json!({
            "type": "noul",
            "instructions": instructions,
            "criteria": { "true": when_true, "false": when_false },
        }),
        Question::Choice {
            instructions,
            options,
            ..
        } => {
            let criteria: Map<String, Value> = options
                .iter()
                .map(|(name, meaning)| ((*name).to_string(), Value::from(*meaning)))
                .collect();
            json!({ "type": "choice", "instructions": instructions, "criteria": criteria })
        }
        Question::Score {
            instructions,
            levels,
            ..
        } => json!({ "type": "score", "instructions": instructions, "criteria": levels }),
    }
}

/// One question's answer, as the model gave it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Answer {
    /// A noul.
    Noul {
        /// The probability of yes, from 0 to 1.
        noul: f64,
    },
    /// A choice.
    Choice {
        /// The option chosen.
        choice: String,
        /// How sure the model is, from 0 to 1.
        confidence: f64,
        /// Each option's probability.
        #[serde(default)]
        probabilities: BTreeMap<String, f64>,
    },
    /// A score.
    Score {
        /// The expected level, fractional.
        score: f64,
        /// How sure the model is, from 0 to 1.
        confidence: f64,
        /// Each level's probability, by the level's number.
        #[serde(default)]
        probabilities: BTreeMap<String, f64>,
        /// What the model said of each level, kept as it came.
        #[serde(default)]
        legend: Value,
    },
}

/// The answers to one request.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Answers {
    /// The model that answered, when it said.
    pub model: Option<String>,
    /// Each answer this layer could read, by its question's key.
    pub by_key: BTreeMap<String, Answer>,
    /// The keys whose answer this layer could not read.
    pub unreadable: Vec<String>,
    /// The answers as they came, for the log.
    pub raw: Value,
    /// The input tokens the call spent.
    pub input_tokens: u64,
}

/// Why an answer's body could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// The body is not JSON; the parser's reason.
    NotJson(String),
    /// The body holds no `answers` object.
    NoAnswers,
    /// Cloudflare's envelope says the run failed; its first error's words.
    Failed(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotJson(reason) => write!(formatter, "the answer is not JSON: {reason}"),
            Self::NoAnswers => formatter.write_str("the answer holds no answers"),
            Self::Failed(reason) => formatter.write_str(reason),
        }
    }
}

impl std::error::Error for ParseError {}

/// Reads an answer's body.
///
/// # Errors
///
/// When the body is not JSON or holds no `answers` object. An answer of a kind this layer does not
/// know is no error: its key is listed in [`Answers::unreadable`].
pub fn parse(body: &str) -> Result<Answers, ParseError> {
    let value: Value =
        serde_json::from_str(body).map_err(|error| ParseError::NotJson(error.to_string()))?;
    parse_value(&value)
}

/// Reads a Workers AI answer: Jev's answer is the envelope's `result`.
///
/// # Errors
///
/// When the body is not JSON, when the envelope says it failed or holds no `result` (its first
/// error's words), or when `result` holds no `answers` object.
pub fn parse_cloudflare(body: &str) -> Result<Answers, ParseError> {
    let envelope: Value =
        serde_json::from_str(body).map_err(|error| ParseError::NotJson(error.to_string()))?;
    let failed = envelope.get("success").and_then(Value::as_bool) == Some(false);
    match envelope.get("result").filter(|result| !result.is_null()) {
        Some(result) if !failed => parse_value(result),
        _ => Err(ParseError::Failed(
            cloudflare_error_of(&envelope)
                .unwrap_or_else(|| "Cloudflare answered with no result".to_string()),
        )),
    }
}

/// The first error of a Workers AI envelope, with its code, when the body is one: what an error
/// status's body says, in Cloudflare's words.
#[must_use]
pub fn cloudflare_error(body: &str) -> Option<String> {
    serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|envelope| cloudflare_error_of(&envelope))
}

fn cloudflare_error_of(envelope: &Value) -> Option<String> {
    let first = envelope.get("errors")?.as_array()?.first()?;
    let message = first.get("message").and_then(Value::as_str)?.trim();
    Some(
        first
            .get("code")
            .and_then(Value::as_i64)
            .map_or_else(|| message.to_string(), |code| format!("{message} ({code})")),
    )
}

fn parse_value(value: &Value) -> Result<Answers, ParseError> {
    let raw = value
        .get("answers")
        .and_then(Value::as_object)
        .ok_or(ParseError::NoAnswers)?;
    let mut by_key = BTreeMap::new();
    let mut unreadable = Vec::new();
    for (key, answer) in raw {
        match Answer::deserialize(answer) {
            Ok(answer) => {
                let _replaced = by_key.insert(key.clone(), answer);
            }
            Err(_) => unreadable.push(key.clone()),
        }
    }
    Ok(Answers {
        model: value
            .get("model")
            .and_then(Value::as_str)
            .map(str::to_string),
        by_key,
        unreadable,
        raw: Value::Object(raw.clone()),
        input_tokens: value
            .pointer("/usage/input_tokens")
            .and_then(Value::as_u64)
            .unwrap_or(0),
    })
}

/// An error's body as it is kept: trimmed and cut to [`ERROR_LIMIT`] characters.
#[must_use]
pub fn error_excerpt(body: &str) -> String {
    cut(body.trim(), ERROR_LIMIT)
}

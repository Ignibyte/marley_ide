//! A plan's open questions (#728): plannotator's `:::question` blocks read out of a plan's
//! Markdown, and the answers written back for the agent.
//!
//! A block opens with a line `:::question` (one choice), `:::question-multi` (any number) or
//! `:::question-text` (free text) and closes with a line `:::`. Its first line is the question;
//! other lines are context; `- [ ] label` is a choice, a reason after ` — ` or ` - `, and `- [x]`
//! one already settled; `Recommended: label` names the agent's pick. A block with no choices takes
//! free text, and one with no closing line, or inside a fenced code block, stays text.

use crate::{PlanNote, plan_feedback};

/// What a question takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestionKind {
    /// One choice.
    One,
    /// Any number of choices.
    Many,
    /// Free text.
    Text,
}

/// One choice of a question.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    /// What the choice says.
    pub label: String,
    /// Why, when the plan says.
    pub reason: Option<String>,
    /// Whether the plan marks it chosen already (`- [x]`).
    pub settled: bool,
}

/// One question of a plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    /// What it takes.
    pub kind: QuestionKind,
    /// The question, its block's first line.
    pub prompt: String,
    /// The block's other lines, as Markdown.
    pub context: String,
    /// Its choices, in the plan's order.
    pub choices: Vec<Choice>,
    /// The choice the agent recommends, by its label.
    pub recommended: Option<String>,
}

/// A run of a plan: Markdown, with where it starts in the plan, or a question.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanPart {
    /// Markdown text, from byte `start` of the plan.
    Text {
        /// Where the run starts in the plan.
        start: usize,
        /// The run.
        text: String,
    },
    /// A question block.
    Question(Question),
}

/// The plan's runs of Markdown and its closed question blocks, in order.
#[must_use]
pub fn plan_parts(source: &str) -> Vec<PlanPart> {
    let lines: Vec<(usize, &str)> = source
        .split_inclusive('\n')
        .scan(0, |offset, line| {
            let start = *offset;
            *offset += line.len();
            Some((start, line))
        })
        .collect();
    let mut parts = Vec::new();
    let mut text = String::new();
    let mut text_start = 0;
    let mut index = 0;
    // The fence a code block opened with, while inside one: an example of the syntax is not a card.
    let mut fence: Option<&str> = None;
    while let Some(&(offset, line)) = lines.get(index) {
        let trimmed = line.trim();
        let marker = ["```", "~~~"]
            .into_iter()
            .find(|marker| trimmed.starts_with(marker));
        match (fence, marker) {
            (None, Some(marker)) => fence = Some(marker),
            (Some(open), Some(marker)) if open == marker => fence = None,
            _ => {}
        }
        if let Some(kind) = opener(trimmed).filter(|_| fence.is_none()) {
            let close = lines
                .iter()
                .enumerate()
                .skip(index + 1)
                .find(|(_, (_, line))| line.trim() == ":::")
                .map(|(close, _)| close);
            if let Some(close) = close {
                if !text.is_empty() {
                    parts.push(PlanPart::Text {
                        start: text_start,
                        text: std::mem::take(&mut text),
                    });
                }
                let body: Vec<&str> = lines
                    .get(index + 1..close)
                    .unwrap_or_default()
                    .iter()
                    .map(|(_, line)| *line)
                    .collect();
                parts.push(PlanPart::Question(question_of(kind, &body)));
                index = close + 1;
                continue;
            }
        }
        if text.is_empty() {
            text_start = offset;
        }
        text.push_str(line);
        index += 1;
    }
    if !text.is_empty() {
        parts.push(PlanPart::Text {
            start: text_start,
            text,
        });
    }
    parts
}

/// The kind a block's opening line names.
fn opener(line: &str) -> Option<QuestionKind> {
    match line {
        ":::question" => Some(QuestionKind::One),
        ":::question-multi" => Some(QuestionKind::Many),
        ":::question-text" => Some(QuestionKind::Text),
        _ => None,
    }
}

/// A block's question, context, choices and pick.
fn question_of(kind: QuestionKind, body: &[&str]) -> Question {
    let mut question: Option<String> = None;
    let mut context: Vec<&str> = Vec::new();
    let mut choices = Vec::new();
    let mut recommended = None;
    for line in body {
        let line = line.trim();
        if line.is_empty() {
            if question.is_some() {
                context.push("");
            }
            continue;
        }
        if let Some(pick) = line.strip_prefix("Recommended:") {
            recommended = Some(pick.trim().to_string());
        } else if let Some((settled, rest)) = choice_line(line) {
            let (label, reason) = rest
                .split_once(" — ")
                .or_else(|| rest.split_once(" - "))
                .map_or((rest, None), |(label, reason)| (label, Some(reason)));
            choices.push(Choice {
                label: label.trim().to_string(),
                reason: reason.map(|reason| reason.trim().to_string()),
                settled,
            });
        } else if question.is_none() {
            question = Some(line.to_string());
        } else {
            context.push(line);
        }
    }
    Question {
        kind: if choices.is_empty() {
            QuestionKind::Text
        } else {
            kind
        },
        prompt: question.unwrap_or_default(),
        context: context.join("\n").trim().to_string(),
        choices,
        recommended,
    }
}

/// A task-list line's mark and its text.
fn choice_line(line: &str) -> Option<(bool, &str)> {
    let rest = line
        .strip_prefix("- ")
        .or_else(|| line.strip_prefix("* "))?;
    rest.strip_prefix("[ ]").map_or_else(
        || {
            rest.strip_prefix("[x]")
                .or_else(|| rest.strip_prefix("[X]"))
                .map(|rest| (true, rest.trim()))
        },
        |rest| Some((false, rest.trim())),
    )
}

/// The user's answer to a question.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    /// The choices picked, and whether that was the agent's recommendation accepted.
    Choices {
        /// The picked choices' labels.
        picked: Vec<String>,
        /// Whether the user accepted the recommendation.
        recommended: bool,
    },
    /// Free text.
    Text(String),
    /// The user's own answer to a question with choices.
    Other(String),
    /// Skipped.
    Skipped,
}

/// The answers' part of a plan's feedback: `Answers to your questions`, the count, each question
/// answered or skipped, then the ones left open. User text has its control characters dropped.
#[must_use]
pub fn plan_answers(questions: &[(String, Option<Answer>)]) -> String {
    let answered = questions
        .iter()
        .filter(|(_, answer)| {
            answer
                .as_ref()
                .is_some_and(|answer| *answer != Answer::Skipped)
        })
        .count();
    let mut sections = vec![format!(
        "Answers to your questions\n{answered} of {} questions answered.",
        questions.len()
    )];
    for (index, (question, answer)) in questions.iter().enumerate() {
        let Some(answer) = answer else {
            continue;
        };
        let body = match answer {
            Answer::Choices {
                picked,
                recommended,
            } => match picked.as_slice() {
                [one] => format!(
                    "Answer: {}{}",
                    clean(one),
                    if *recommended {
                        " (your recommendation)"
                    } else {
                        ""
                    }
                ),
                several => format!(
                    "Answer:\n{}",
                    several
                        .iter()
                        .map(|choice| format!("- {}", clean(choice)))
                        .collect::<Vec<_>>()
                        .join("\n")
                ),
            },
            Answer::Text(text) => format!(
                "Answer:\n{}",
                clean(text)
                    .lines()
                    .map(|line| format!("> {line}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
            Answer::Other(words) => format!("Other: {}", clean(words)),
            Answer::Skipped => "Skipped".to_string(),
        };
        sections.push(format!(
            "### Question {}: {}\n{body}",
            index + 1,
            clean(question)
        ));
    }
    let open: Vec<String> = questions
        .iter()
        .filter(|(_, answer)| answer.is_none())
        .map(|(question, _)| format!("- {}", clean(question)))
        .collect();
    if !open.is_empty() {
        sections.push(format!("Unanswered:\n{}", open.join("\n")));
    }
    sections.join("\n\n")
}

/// A plan's feedback with answers (#728): the answers first, then the notes in #726's form; with no
/// notes, a lead line of its own.
#[must_use]
pub fn questions_feedback(answers: &str, notes: &[PlanNote], overall: &str) -> String {
    if notes.is_empty() && overall.trim().is_empty() {
        format!(
            "The user answered your questions. Revise the plan with these answers, then call \
             ExitPlanMode again.\n\n{answers}"
        )
    } else {
        format!("{answers}\n\n{}", plan_feedback(notes, overall))
    }
}

/// `text` trimmed, its control characters other than a line break dropped.
fn clean(text: &str) -> String {
    text.chars()
        .filter(|character| !character.is_control() || *character == '\n')
        .collect::<String>()
        .trim()
        .to_string()
}

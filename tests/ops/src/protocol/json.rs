//! Bounded JSON input for ops tools.

//! Bounded JSON-object decoding with Skein's tokenizer. The decoded tree is
//! temporary and never crosses the engine or Smith domain boundary.

use skein_json::{Token, tokenizer};
use skein_lib::stream::{Down, Up};
use skein_lib::{Env, Intake, List, Queue, Stack, Time, Wall};
use std::boxed::Box;

use jig_protocol_tools::Why as Problem;

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum Value {
    Object(Box<[(Box<[u8]>, Value)]>),
    Array(Box<[Value]>),
    String(Box<[u8]>),
    Number(Box<[u8]>),
    Boolean(bool),
    Null,
}

pub(crate) fn parse(input: &[u8]) -> Result<Value, Problem> {
    if input.len() > 65_536 {
        return Err(Problem::TooLarge);
    }
    let Ok(length) = u32::try_from(input.len()) else { return Err(Problem::TooLarge) };
    let limits = tokenizer::Limits { depth: 16, string: length, number: 24, chunk: length.max(1), length };
    let mut machine = tokenizer::Tokenizer::new(&limits);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut intake = Intake::with_capacity(length.max(tokenizer::largest_demand(&limits)));
    if intake.append(input).is_err() {
        return Err(Problem::TooLarge);
    }
    let mut above = Queue::with_capacity(1);
    let mut below = Queue::with_capacity(1);
    let mut tokens = List::with_capacity(4096);
    let mut demanded = None;
    let steps = input.len().checked_mul(4).ok_or(Problem::TooLarge)?;
    let steps = steps.checked_add(8).ok_or(Problem::TooLarge)?;
    for _ in 0..steps {
        match demanded.take() {
            Some(read) => match intake.meet(read) {
                Some(bytes) => tokenizer::up(&mut machine, &env, Up::Bytes(bytes), &mut above, &mut below),
                None => tokenizer::up(&mut machine, &env, Up::End, &mut above, &mut below),
            },
            None => tokenizer::down(&mut machine, &env, tokenizer::Request::Next, &mut above, &mut below),
        }
        match below.pop() {
            Some(Down::Demand { read, .. }) => demanded = Some(read),
            Some(Down::Send(_) | Down::Finish) => return Err(Problem::Malformed),
            None => {}
        }
        match above.pop() {
            Some(tokenizer::Event::Token(token)) => {
                let Ok(()) = tokens.push(token) else { return Err(Problem::TooLarge) };
            }
            Some(tokenizer::Event::Done) => {
                let tokens = tokens.into_boxed();
                let value = value(tokens)?;
                return match value {
                    Value::Object(_) => Ok(value),
                    Value::Array(_) | Value::String(_) | Value::Number(_) | Value::Boolean(_) | Value::Null => {
                        Err(Problem::Type)
                    }
                };
            }
            // Only `Next` is asked, so a capped text's or a skip's answer never comes.
            Some(
                tokenizer::Event::Failed(_)
                | tokenizer::Event::Closed
                | tokenizer::Event::Long(_)
                | tokenizer::Event::Skipped(_),
            ) => return Err(Problem::Malformed),
            None => {}
        }
    }
    Err(Problem::Malformed)
}

enum Frame {
    Object { fields: List<(Box<[u8]>, Value)>, key: Option<Box<[u8]>> },
    Array(List<Value>),
}

fn value(tokens: Box<[Token]>) -> Result<Value, Problem> {
    let mut stack = Stack::with_capacity(16);
    let mut result = None;
    for token in tokens {
        let item = match token {
            Token::ObjectStart => {
                if stack.push(Frame::Object { fields: List::with_capacity(128), key: None }).is_err() {
                    return Err(Problem::TooLarge);
                }
                continue;
            }
            Token::ArrayStart => {
                if stack.push(Frame::Array(List::with_capacity(128))).is_err() {
                    return Err(Problem::TooLarge);
                }
                continue;
            }
            Token::Key(next) => {
                let Some(Frame::Object { fields, key }) = stack.top_mut() else { return Err(Problem::Malformed) };
                if key.is_some() {
                    return Err(Problem::Malformed);
                }
                for (previous, _) in fields.as_slice() {
                    if previous == &next {
                        return Err(Problem::Malformed);
                    }
                }
                *key = Some(next);
                continue;
            }
            Token::ObjectEnd => match stack.pop() {
                Some(Frame::Object { fields, key: None }) => Value::Object(fields.into_boxed()),
                Some(Frame::Object { key: Some(_), .. } | Frame::Array(_)) | None => {
                    return Err(Problem::Malformed);
                }
            },
            Token::ArrayEnd => match stack.pop() {
                Some(Frame::Array(items)) => Value::Array(items.into_boxed()),
                Some(Frame::Object { .. }) | None => return Err(Problem::Malformed),
            },
            Token::String(text) => Value::String(text),
            Token::Number(text) => Value::Number(text),
            Token::True => Value::Boolean(true),
            Token::False => Value::Boolean(false),
            Token::Null => Value::Null,
        };
        match stack.top_mut() {
            Some(Frame::Object { fields, key }) => {
                let key = key.take().ok_or(Problem::Malformed)?;
                if fields.push((key, item)).is_err() {
                    return Err(Problem::TooLarge);
                }
            }
            Some(Frame::Array(items)) => {
                if items.push(item).is_err() {
                    return Err(Problem::TooLarge);
                }
            }
            None => {
                if result.is_some() {
                    return Err(Problem::Malformed);
                }
                result = Some(item);
            }
        }
    }
    if !stack.is_empty() {
        return Err(Problem::Malformed);
    }
    result.ok_or(Problem::Malformed)
}

impl Value {
    pub(crate) fn field(&self, name: &[u8]) -> Result<Option<&Value>, Problem> {
        match self {
            Value::Object(fields) => {
                for (key, value) in fields {
                    if key.as_ref() == name {
                        return Ok(Some(value));
                    }
                }
                Ok(None)
            }
            Value::Array(_) | Value::String(_) | Value::Number(_) | Value::Boolean(_) | Value::Null => {
                Err(Problem::Type)
            }
        }
    }

    pub(crate) fn required(&self, name: &[u8]) -> Result<&Value, Problem> {
        self.field(name)?.ok_or(Problem::Missing)
    }

    pub(crate) fn text(&self) -> Result<Box<[u8]>, Problem> {
        match self {
            Value::String(text) => Ok(text.clone()),
            Value::Object(_) | Value::Array(_) | Value::Number(_) | Value::Boolean(_) | Value::Null => {
                Err(Problem::Type)
            }
        }
    }

    pub(crate) fn number(&self) -> Result<u64, Problem> {
        match self {
            Value::Number(text) => super::decimal(text).ok_or(Problem::Range),
            Value::Object(_) | Value::Array(_) | Value::String(_) | Value::Boolean(_) | Value::Null => {
                Err(Problem::Type)
            }
        }
    }

    pub(crate) fn small(&self) -> Result<u32, Problem> {
        let Ok(value) = u32::try_from(self.number()?) else { return Err(Problem::Range) };
        Ok(value)
    }
}

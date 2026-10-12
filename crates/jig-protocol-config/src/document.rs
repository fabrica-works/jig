//! Pump the whole bounded input through skein's JSON tokenizer (json.md, 3 and 5.2).
use crate::{Path, Problem, What};
use skein_json::{Document, document, tokenizer};
use skein_lib::stream::{Down, Up};
use skein_lib::{Env, Intake, List, Queue, Time, Wall};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DocumentLimits {
    pub bytes: u32,
    pub tokens: u32,
    pub depth: u32,
}
pub fn document(input: &[u8], bounds: &DocumentLimits) -> Result<Document, Problem> {
    let problem = Problem { path: Path::root(), what: What::Json };
    let length = u32::try_from(input.len()).or(Err(problem.clone()))?;
    if length > bounds.bytes {
        return Err(problem);
    }
    let limits =
        tokenizer::Limits { depth: bounds.depth, string: length, number: length, chunk: length.max(1), length };
    let mut machine = tokenizer::Tokenizer::new(&limits);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut intake = Intake::with_capacity(length.max(tokenizer::largest_demand(&limits)));
    intake.append(input).or(Err(problem.clone()))?;
    let mut above = Queue::with_capacity(1);
    let mut below = Queue::with_capacity(1);
    let mut tokens = List::with_capacity(bounds.tokens);
    let mut demanded = None;
    let steps =
        u64::from(length).checked_mul(4).expect("u32 input fits step bound").checked_add(8).expect("step bound fits");
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
            Some(Down::Send(_) | Down::Finish) => return Err(problem),
            None => {}
        }
        match above.pop() {
            Some(tokenizer::Event::Token(token)) => {
                tokens.push(token).or(Err(problem.clone()))?;
            }
            Some(tokenizer::Event::Done) => {
                let limits = document::Limits { tokens: bounds.tokens, text: bounds.bytes };
                return Document::from_tokens(tokens.as_slice(), &limits).or(Err(problem));
            }
            Some(
                tokenizer::Event::Failed(_)
                | tokenizer::Event::Closed
                | tokenizer::Event::Long(_)
                | tokenizer::Event::Skipped(_),
            ) => return Err(problem),
            None => {}
        }
    }
    Err(problem)
}

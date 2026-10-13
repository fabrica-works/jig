//! Reusable channel scenarios through the real reference services.
use crate::{
    fixtures,
    world::{self, Outcome},
};
use skein_lib::Duration;

#[derive(Clone, Copy, Debug)]
pub enum Story {
    Chat,
    Stalled,
    Quiet,
    Replaced,
    Reconnect,
    BeyondGrace,
    Shutdown,
    Refusals(MessageRefusal),
    Opening(Opening),
    Progress,
    Call,
    Facts,
    Inbox,
    Wait,
    Window,
    InvalidTranscript,
    MidFrame,
    InvalidCharter,
    Restart { before: u32 },
    HeldCommit,
    Partition { engine: Duration, worker: Duration },
}

#[must_use]
pub fn run(seed: u64, story: Story) -> Outcome {
    world::run(fixtures::calm(seed), script(story))
}

fn script(story: Story) -> crate::world::Script {
    let mut script = fixtures::one_chat(b"Hello");
    script.story = story;
    if matches!(story, Story::Progress | Story::Call | Story::Wait | Story::InvalidTranscript | Story::Refusals(_)) {
        script.leading_turns = 1;
    }
    if matches!(story, Story::Restart { .. }) {
        script.latency = Duration::from_secs(1);
    }
    if matches!(story, Story::Partition { .. } | Story::Facts | Story::Reconnect | Story::Shutdown | Story::Refusals(_))
    {
        script.latency = Duration::from_secs(1);
    }
    if matches!(story, Story::Refusals(MessageRefusal::Ending | MessageRefusal::Full | MessageRefusal::NameInUse)) {
        script.leading_turns = 0;
        script.latency = Duration::ZERO;
    }
    if matches!(story, Story::BeyondGrace) {
        script.latency = Duration::from_secs(25);
    }
    script
}

#[must_use]
pub fn drawn(seed: u64) -> Outcome {
    let mut draw = skein_lib::Rng::new(seed);
    let mut settings = fixtures::calm(seed);
    settings.config = skein_sim::Config::chaos();
    settings.config.buffer = u32::try_from(draw.between(256, 4096)).expect("small buffer");
    settings.config.faults = skein_sim::Faults {
        short_recv: u32::try_from(draw.below(600)).expect("short receive probability"),
        short_send: u32::try_from(draw.below(600)).expect("short send probability"),
        latency: 200,
        latency_max: Duration::from_millis(1),
        ..skein_sim::Faults::NONE
    };
    let story = match draw.below(6) {
        0 => Story::Chat,
        1 => Story::Progress,
        2 => Story::Call,
        3 => Story::HeldCommit,
        4 => Story::MidFrame,
        5 => Story::Restart { before: 20 },
        _ => panic!("bounded draw"),
    };
    world::run(settings, script(story))
}

#[derive(Clone, Copy, Debug)]
pub enum Opening {
    Secret,
    Family,
    Version,
    Application,
    Terms,
    Garbage,
}

#[derive(Clone, Copy, Debug)]
pub enum MessageRefusal {
    TooLarge,
    Full,
    NameInUse,
    Ending,
    Replay,
}

#[must_use]
pub fn refusal(seed: u64, reason: MessageRefusal, recorded: crate::world::Recorded) -> Outcome {
    let mut script = script(Story::Refusals(reason));
    script.recorded = Some(recorded);
    world::run(fixtures::calm(seed), script)
}

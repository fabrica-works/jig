//! The reference service and its agent command (protocol/service.md, 7).
use std::{env, process::ExitCode};

fn main() -> ExitCode {
    match jig_test::configure(env::args_os().skip(1)).and_then(jig_test::run) {
        Ok(code) => ExitCode::from(code),
        Err(why) => {
            eprintln!("jig-test: {why}");
            ExitCode::FAILURE
        }
    }
}

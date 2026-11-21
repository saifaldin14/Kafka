//! `cargo xtask tools install`: installs the pinned external tools that are missing or
//! at the wrong version, using the step runner.

use std::io::Write;

use crate::Exit;
use crate::output;
use crate::steps::{self, Step};
use crate::tools::{self, TOOLS};

pub(crate) fn run(out: &mut impl Write) -> Exit {
    let installs: Vec<Step> = TOOLS
        .iter()
        .filter(|tool| tools::check(tool).is_err())
        .map(|tool| {
            Step::cargo(
                &format!("install {}", tool.crate_name),
                &[
                    "install",
                    "--locked",
                    "--version",
                    tool.version,
                    tool.crate_name,
                ],
            )
        })
        .collect();
    if installs.is_empty() {
        output::line(out, "All tools are installed at their pinned versions.");
        return Exit::Success;
    }
    if steps::all_passed(&steps::run(&installs, false, out)) {
        Exit::Success
    } else {
        Exit::CheckFailed
    }
}

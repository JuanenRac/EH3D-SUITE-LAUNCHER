//! Starting a program of the suite and keeping it alive: a program that dies with an error is started again a limited number of times, one that ends well is left alone.

use std::path::Path;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Policy {
    /// How many times a program that failed is started again (0 = never).
    pub max_restarts: u32,
    pub backoff_ms: u64,
}

impl Default for Policy {
    fn default() -> Self {
        Policy {
            max_restarts: 2,
            backoff_ms: 500,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// How many times the program was started.
    pub runs: u32,
    /// The exit code of the last run; `None` when it was ended by a signal.
    pub last_code: Option<i32>,
    /// True when the last run failed and the restarts were used up.
    pub gave_up: bool,
}

/// Runs `program` with `args` and waits. A failed run (a code other than 0, or a signal) is repeated up to `policy.max_restarts` times, with the waiting time doubling.
pub fn supervise(program: &Path, args: &[String], policy: Policy) -> Result<Outcome, String> {
    let mut runs = 0;
    let mut wait = policy.backoff_ms;
    loop {
        runs += 1;
        let status = Command::new(program)
            .args(args)
            .status()
            .map_err(|e| format!("{}: {e}", program.display()))?;
        let code = status.code();
        if code == Some(0) {
            return Ok(Outcome {
                runs,
                last_code: code,
                gave_up: false,
            });
        }
        if runs > policy.max_restarts {
            return Ok(Outcome {
                runs,
                last_code: code,
                gave_up: true,
            });
        }
        sleep(Duration::from_millis(wait));
        wait = wait.saturating_mul(2).min(30_000);
    }
}

//! The arguments of the window.
//!
//! Pattern: none (a plain parser) — `--store <file>` and `--help`; no other options exist.

use std::path::PathBuf;

/// The help text.
pub const HELP: &str = "fs-whiteboard [app] [--store <file>]\n  --store <file>   keep the boards in this file (default: the data folder)\n  --help           this text";

/// What was asked for.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Args {
    /// Keep the store in this file.
    pub store: Option<PathBuf>,
    /// Print the help and stop.
    pub help: bool,
}

/// Read the words after `app`.
///
/// # Errors
/// A sentence for the person at the command line when a word is unknown or a value is missing.
pub fn parse(words: &[String]) -> Result<Args, String> {
    let mut args = Args::default();
    let mut it = words.iter();
    while let Some(word) = it.next() {
        match word.as_str() {
            "--help" | "-h" => args.help = true,
            "--store" => {
                let file = it.next().ok_or("--store needs a file name")?;
                args.store = Some(PathBuf::from(file));
            }
            other => return Err(format!("unknown option `{other}`")),
        }
    }
    Ok(args)
}

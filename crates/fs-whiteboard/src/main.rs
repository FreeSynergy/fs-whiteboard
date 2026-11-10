//! `fs-whiteboard`: the one program of FreeSynergy.Whiteboard (ADR 0063).
//!
//! Pattern: Command — the first word picks the subcommand. Today there is one: `app` (or no word at all) opens the
//! window (`fs-whiteboard-app`, feature `gui`). There is no `serve` and no command line yet: the core has no capnp
//! server and no command line (`DESIGN.md`, Unsupported). Without the feature `gui` the bare command prints the help,
//! so a machine without a screen does not hang on graphics libraries.

fn main() {
    let words: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(dispatch(&words));
}

fn dispatch(words: &[String]) -> i32 {
    match words.split_first() {
        Some((first, rest)) if first == "app" => window(rest),
        Some((first, _)) if first == "--help" || first == "-h" || first == "help" => {
            println!("{HELP}");
            0
        }
        None => window(&[]),
        Some((first, _)) => {
            eprintln!("fs-whiteboard: unknown command `{first}`\n{HELP}");
            2
        }
    }
}

const HELP: &str = "fs-whiteboard [app] [--store <file>]\n  app    open the window (default)\n  --store <file>   keep the boards in this file (default: the data folder)";

#[cfg(feature = "gui")]
fn window(args: &[String]) -> i32 {
    fs_whiteboard_app::app_command(args)
}

#[cfg(not(feature = "gui"))]
fn window(args: &[String]) -> i32 {
    if args.is_empty() {
        println!("{HELP}");
        0
    } else {
        eprintln!("fs-whiteboard: this build has no window (feature gui)");
        2
    }
}

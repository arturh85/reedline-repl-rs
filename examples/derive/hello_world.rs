//! Minimal example
use std::collections::HashMap;

use reedline_repl_rs::clap::{ArgMatches, Parser};
use reedline_repl_rs::{CallBackMap, Repl, Result};

#[derive(Parser, Debug)]
#[command(name = "MyApp", version = "v0.1.0", about = "My very cool app")]
pub enum MyApp {
    /// Greeting
    Hello { who: String },
}

/// Write "Hello" with given name
fn hello<T>(args: ArgMatches, _context: &mut T) -> Result<Option<String>> {
    Ok(Some(format!(
        "Hello, {}",
        args.get_one::<String>("who").unwrap()
    )))
}

fn main() -> Result<()> {
    let mut callbacks: CallBackMap<(), reedline_repl_rs::Error> = HashMap::new();

    callbacks.insert("hello".to_string(), hello);

    let mut repl = Repl::new(())
        .with_banner("Welcome to MyApp")
        .with_derived::<MyApp>(callbacks);

    repl.run()
}

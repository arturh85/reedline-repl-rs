//! Example using Repl without Context (or, more precisely, a Context of ())
use std::collections::HashMap;

use clap::{Parser, Subcommand};
use reedline_repl_rs::clap::ArgMatches;
use reedline_repl_rs::{CallBackMap, Repl, Result};

#[derive(Parser, Debug)]
#[command(name = "MyApp", version = "v0.1.0", about = "My very cool List")]
pub struct MyApp {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Add two numbers together
    Add { first: i32, second: i32 },
    /// Greetings!
    Hello { who: String },
}

/// Add two numbers. Have to make this generic to be able to pass a Context of type ()
fn add<T>(args: ArgMatches, _context: &mut T) -> Result<Option<String>> {
    let first = args.get_one::<i32>("first").unwrap();
    let second = args.get_one::<i32>("second").unwrap();

    Ok(Some((first + second).to_string()))
}

/// Write "Hello"
fn hello<T>(args: ArgMatches, _context: &mut T) -> Result<Option<String>> {
    Ok(Some(format!(
        "Hello, {}",
        args.get_one::<String>("who").unwrap()
    )))
}

fn main() -> Result<()> {
    let mut callbacks: CallBackMap<(), reedline_repl_rs::Error> = HashMap::new();
    callbacks.insert("add".to_string(), add);
    callbacks.insert("hello".to_string(), hello);

    let mut repl = Repl::new(()).with_derived::<MyApp>(callbacks);

    repl.run()
}

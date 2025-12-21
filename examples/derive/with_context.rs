//! Example using Repl with Context
use clap::{Parser, Subcommand};
use reedline_repl_rs::clap::ArgMatches;
use reedline_repl_rs::{CallBackMap, Repl, Result};
use std::collections::{HashMap, VecDeque};

#[derive(Parser, Debug)]
#[command(name = "MyApp", version = "v0.1.0", about = "My very cool List")]
pub struct MyList {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Append name to end of list
    Append { name: String },
    /// Prepend name to front of list
    Prepend { name: String },
}

#[derive(Default)]
struct Context {
    list: VecDeque<String>,
}

/// Append name to list
fn append(args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    let name: String = args.get_one::<String>("name").unwrap().to_string();
    context.list.push_back(name);
    let list: Vec<String> = context.list.clone().into();

    Ok(Some(list.join(", ")))
}

/// Prepend name to list
fn prepend(args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    let name: String = args.get_one::<String>("name").unwrap().to_string();
    context.list.push_front(name);
    let list: Vec<String> = context.list.clone().into();

    Ok(Some(list.join(", ")))
}

fn main() -> Result<()> {
    let mut callbacks: CallBackMap<Context, reedline_repl_rs::Error> = HashMap::new();

    callbacks.insert("append".to_string(), append);
    callbacks.insert("prepend".to_string(), prepend);

    let mut repl = Repl::new(Context::default())
        .with_derived::<MyList>(callbacks)
        .with_on_after_command(|context| Ok(Some(format!("MyList [{}]", context.list.len()))));

    repl.run()
}

//! Minimal example
use std::collections::HashMap;

use reedline_repl_rs::clap::{ArgMatches, Parser};
use reedline_repl_rs::{AsyncCallBackMap, Repl, Result};

#[derive(Parser, Debug)]
#[command(name = "MyApp", version = "v0.1.0", about = "My very cool app")]
pub enum MyApp {
    /// Greeting
    Hello { who: String },
}

/// Write "Hello" with given name
async fn hello<T>(args: ArgMatches, _context: &mut T) -> Result<Option<String>> {
    Ok(Some(format!(
        "Hello, {}",
        args.get_one::<String>("who").unwrap()
    )))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut callbacks: AsyncCallBackMap<(), reedline_repl_rs::Error> = HashMap::new();

    callbacks.insert("hello".to_string(), |args, context| {
        Box::pin(hello(args, context))
    });

    let mut repl = Repl::new(())
        .with_name("MyApp")
        .with_version("v0.1.0")
        .with_async_derived::<MyApp>(callbacks);

    repl.run_async().await
}

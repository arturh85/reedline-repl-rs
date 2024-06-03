# reedline-repl-rs

Library to help you create a fancy [REPL](https://en.wikipedia.org/wiki/Read%E2%80%93eval%E2%80%93print_loop) for your application based on [nushell](https://github.com/nushell/nushell)'s [reedline](https://github.com/nushell/reedline).

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/reedline-repl-rs.svg)](https://crates.io/crates/reedline-repl-rs)
[![Documentation](https://docs.rs/reedline-repl-rs/badge.svg)](https://docs.rs/reedline-repl-rs/latest/)

Features:
- Popular [clap](https://github.com/clap-rs/clap) crate [Command](https://docs.rs/clap/latest/clap/type.Command.html) used as configuration interface
- Also supports [clap-derive](https://docs.rs/clap/latest/clap/_derive/) based configuration with feature flag, see [derive examples](https://github.com/arturh85/reedline-repl-rs/tree/main/examples/derive)
- General editing functionality, that should feel familiar coming from other shells (e.g. bash, fish, zsh).
- Interactive tab-completion with graphical selection menu 
- Fish-style history autosuggestion hints
- History with interactive search options (optionally persists to file, can support multiple sessions accessing the same file)
- Configurable keybindings (default emacs-style bindings).
- Configurable prompt with hooks to update after commands run
- Command Syntax highlighting 
- Feature-flag for async support
- Feature-flag `shlex` for optional POSIX compliant line splitting
- Tip: Search history with `CTRL+R`, clear input with `CTRL+C`, exit repl with `CTRL+D` 

Basic example code:

```rust
//! Minimal example
use reedline_repl_rs::clap::{Arg, ArgMatches, Command};
use reedline_repl_rs::{Repl, Result};

/// Write "Hello" with given name
fn hello<T>(args: ArgMatches, _context: &mut T) -> Result<Option<String>> {
    Ok(Some(format!(
        "Hello, {}",
        args.get_one::<String>("who").unwrap()
    )))
}

fn main() -> Result<()> {
    let mut repl = Repl::new(())
        .with_name("MyApp")
        .with_version("v0.1.0")
        .with_description("My very cool app")
        .with_banner("Welcome to MyApp")
        .with_command(
            Command::new("hello")
                .arg(Arg::new("who").required(true))
                .about("Greetings!"),
            hello,
        );
    repl.run()
}
```

Running the example above:

![Colored Terminal Output](screenshot.png)
```plain
Welcome to MyApp
MyApp〉help
MyApp v0.1.0: My very cool app

COMMANDS:
    hello    Greetings!
    help     Print this message or the help of the given subcommand(s)

MyApp〉help hello
hello
Greetings!

USAGE:
    hello <who>

ARGS:
    <who>

OPTIONS:
    -h, --help    Print help information
MyApp〉hello Friend
Hello, Friend
MyApp〉
```

## Testing

``` shell
cargo test --features async
```

Will run the doc tests (compiling the examples). Notice, the `examples/async.rs` requires the `async` feature.

## Thanks

Forked from [repl-rs](https://github.com/jacklund/repl-rs) by [Jacklund](https://github.com/jacklund), 
changed to use [reedline](https://github.com/nushell/reedline) which is an advanced readline clone
and the base of [nushell](https://github.com/nushell/nushell).


use crate::command::ReplCommand;
use clap::builder::StyledStr;
use clap::Command;
use nu_ansi_term::Style;
use reedline::{Completer, Span, Suggestion};
use std::collections::HashMap;

pub(crate) struct ReplCompleter {
    commands: HashMap<String, Command>,
}

impl Completer for ReplCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        let mut completions = vec![];
        completions.extend(if line.contains(' ') {
            let words: Vec<&str> = line[0..pos].split(' ').collect();

            // Find the "deepest" subcommand in the line
            let mut deepest_command: Option<&Command> = None;
            let mut deepest_command_idx = 0;
            for (i, word) in words.iter().enumerate() {
                // If we've found a command already, use it to to find subcommands
                if let Some(nearest) = deepest_command {
                    if let Some(subcommand) = nearest.find_subcommand(word) {
                        deepest_command = Some(subcommand);
                        deepest_command_idx = i;
                    }
                } else {
                    // If no command is found, look for a top-level one
                    deepest_command = self.commands.get(*word);
                    deepest_command_idx = i;
                }
            }

            let words_left = &words[deepest_command_idx..];
            let mut words_rev = words_left.iter().rev();

            if let Some(command) = deepest_command {
                let last_word = words_rev.next().unwrap();
                let last_word_start_pos = line.len() - last_word.len();
                let span = Span::new(last_word_start_pos, pos);
                self.parameter_values_starting_with(command, words_rev.count(), last_word, span)
            } else {
                vec![]
            }
        } else {
            let span = Span::new(0, pos);
            self.commands_starting_with(line, span)
        });
        completions.dedup();
        completions
    }
}

impl ReplCompleter {
    pub fn new<Context, E>(repl_commands: &HashMap<String, ReplCommand<Context, E>>) -> Self {
        let mut commands = HashMap::new();
        for (name, repl_command) in repl_commands.iter() {
            commands.insert(name.clone(), repl_command.command.clone());
        }
        ReplCompleter { commands }
    }

    fn build_suggestion(&self, value: &str, help: Option<&StyledStr>, span: Span, style: Option<Style>) -> Suggestion {
        Suggestion {
            value: value.to_string(),
            description: help.map(|n| format!("{}", n)),
            extra: None,
            span,
            style,
            append_whitespace: true,
            match_indices: None,
        }
    }

    fn parameter_values_starting_with(
        &self,
        command: &Command,
        _parameter_idx: usize,
        search: &str,
        span: Span,
    ) -> Vec<Suggestion> {
        let mut completions = vec![];
        for arg in command.get_arguments() {
            // skips --help and --version
            if arg.is_global_set() {
                continue;
            }

            completions.extend(
                arg.get_possible_values()
                    .iter()
                    .filter(|value| value.get_name().starts_with(search))
                    .map(|value| self.build_suggestion(value.get_name(), value.get_help(), span, None)),
            );

            if let Some(long) = arg.get_long() {
                let value = "--".to_string() + long;
                if value.starts_with(search) {
                    completions.push(self.build_suggestion(&value, arg.get_help(), span, None));
                }
            }

            if let Some(short) = arg.get_short() {
                let value = "-".to_string() + &short.to_string();
                if value.starts_with(search) {
                    completions.push(self.build_suggestion(&value, arg.get_help(), span, None));
                }
            }
        }

        for subcommand in command.get_subcommands() {
            if subcommand.get_name().starts_with(search) {
                completions.push(self.build_suggestion(
                    subcommand.get_name(),
                    subcommand.get_after_help(),
                    span, 
                    None,
                ));
            }
        }

        completions
    }

    fn commands_starting_with(&self, search: &str, span: Span) -> Vec<Suggestion> {
        let mut result: Vec<Suggestion> = self
            .commands
            .iter()
            .filter(|(key, _)| key.starts_with(search))
            .map(|(_, command)| {
                self.build_suggestion(command.get_name(), command.get_about(), span, None)
            })
            .collect();

        if "help".starts_with(search) {
            let help: StyledStr = "show help".into();
            result.push(self.build_suggestion("help", Some(&help), span, None));
        }

        result
    }
}

use crate::command::ReplCommand;
use crate::completer::ReplCompleter;
use crate::error::*;
use crate::prompt::ReplPrompt;
use crate::{paint_green_bold, paint_yellow_bold, AfterCommandCallback, Callback};
#[cfg(feature = "async")]
use crate::{AsyncAfterCommandCallback, AsyncCallback};
use clap::Command;
use reedline::MenuBuilder;
// use crossterm::event::{KeyCode, KeyModifiers};
use nu_ansi_term::{Color, Style};
use reedline::{
    self, default_emacs_keybindings, ColumnarMenu, DefaultHinter, DefaultValidator, Emacs,
    ExampleHighlighter, ExternalPrinter, FileBackedHistory, KeyCode, KeyModifiers, Keybindings,
    Reedline, ReedlineEvent, ReedlineMenu, Signal,
};
use std::boxed::Box;
use std::collections::HashMap;
use std::fmt::Display;
use std::path::PathBuf;

type ErrorHandler<Context, E> = fn(error: E, repl: &Repl<Context, E>) -> Result<()>;

fn default_error_handler<Context, E: Display>(error: E, _repl: &Repl<Context, E>) -> Result<()> {
    eprintln!("{}", error);
    Ok(())
}

/// Main REPL struct
pub struct Repl<Context, E: Display> {
    name: String,
    banner: Option<String>,
    version: String,
    description: String,
    prompt: ReplPrompt,
    after_command_callback: Option<AfterCommandCallback<Context, E>>,
    #[cfg(feature = "async")]
    after_command_callback_async: Option<AsyncAfterCommandCallback<Context, E>>,
    commands: HashMap<String, ReplCommand<Context, E>>,
    history: Option<PathBuf>,
    history_capacity: Option<usize>,
    context: Context,
    keybindings: Keybindings,
    external_printer: ExternalPrinter<String>,
    hinter_style: Style,
    hinter_enabled: bool,
    quick_completions: bool,
    partial_completions: bool,
    stop_on_ctrl_c: bool,
    stop_on_ctrl_d: bool,
    error_handler: ErrorHandler<Context, E>,
    #[cfg(windows)]
    restore_stdout_mode: Option<u32>,
    #[cfg(windows)]
    restore_stderr_mode: Option<u32>,
}

impl<Context, E> Repl<Context, E>
where
    E: Display + From<Error> + std::fmt::Debug,
{
    /// Create a new Repl with the given context's initial value.
    pub fn new(context: Context) -> Self {
        let name = String::from("repl");
        let style = Style::new().italic().fg(Color::LightGray);
        let mut keybindings = default_emacs_keybindings();
        keybindings.add_binding(
            KeyModifiers::NONE,
            KeyCode::Tab,
            ReedlineEvent::Menu("completion_menu".to_string()),
        );
        let prompt = ReplPrompt::new(&paint_green_bold(&format!("{}> ", name)));

        Self {
            name,
            banner: None,
            version: String::new(),
            description: String::new(),
            commands: HashMap::new(),
            history: None,
            history_capacity: None,
            after_command_callback: None,
            #[cfg(feature = "async")]
            after_command_callback_async: None,
            quick_completions: true,
            partial_completions: false,
            hinter_enabled: true,
            hinter_style: style,
            prompt,
            context,
            keybindings,
            external_printer: ExternalPrinter::new(2048),
            stop_on_ctrl_c: false,
            stop_on_ctrl_d: true,
            error_handler: default_error_handler,
            #[cfg(windows)]
            restore_stdout_mode: None,
            #[cfg(windows)]
            restore_stderr_mode: None,
        }
    }

    /// Give your Repl a name. This is used in the help summary for the Repl.
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self.with_formatted_prompt(name)
    }

    /// Give your Repl a banner. This is printed at the start of running the Repl.
    pub fn with_banner(mut self, banner: &str) -> Self {
        self.banner = Some(banner.to_string());

        self
    }

    /// Give your Repl a version. This is used in the help summary for the Repl.
    pub fn with_version(mut self, version: &str) -> Self {
        self.version = version.to_string();

        self
    }

    /// Give your Repl a description. This is used in the help summary for the Repl.
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();

        self
    }

    /// Give your REPL a callback which is called after every command and may update the prompt
    pub fn with_on_after_command(mut self, callback: AfterCommandCallback<Context, E>) -> Self {
        self.after_command_callback = Some(callback);

        self
    }

    /// Give your REPL a callback which is called after every command and may update the prompt
    #[cfg(feature = "async")]
    pub fn with_on_after_command_async(
        mut self,
        callback: AsyncAfterCommandCallback<Context, E>,
    ) -> Self {
        self.after_command_callback_async = Some(callback);

        self
    }

    /// Give your Repl a file based history saved at history_path
    pub fn with_history(mut self, history_path: PathBuf, capacity: usize) -> Self {
        self.history = Some(history_path);
        self.history_capacity = Some(capacity);

        self
    }

    /// Give your Repl a custom prompt. The default prompt is the Repl name, followed by
    /// a `>`, all in green and bold, followed by a space:
    ///
    /// &Paint::green(format!("{}> ", name)).bold().to_string()
    pub fn with_prompt(mut self, prompt: &str) -> Self {
        self.prompt.update_prefix(prompt);

        self
    }

    /// Give your Repl a custom prompt while applying green/bold formatting automatically
    ///
    /// &Paint::green(format!("{}> ", name)).bold().to_string()
    pub fn with_formatted_prompt(mut self, prompt: &str) -> Self {
        self.prompt.update_prefix(prompt);

        self
    }

    /// Pass in a custom error handler. This is really only for testing - the default
    /// error handler simply prints the error to stderr and then returns
    pub fn with_error_handler(mut self, handler: ErrorHandler<Context, E>) -> Self {
        self.error_handler = handler;

        self
    }

    /// Turn on/off if REPL run is stopped on CTRG+C (Default: false)
    pub fn with_stop_on_ctrl_c(mut self, stop_on_ctrl_c: bool) -> Self {
        self.stop_on_ctrl_c = stop_on_ctrl_c;

        self
    }

    /// Turn on/off if REPL run is stopped on CTRG+D (Default: true)
    pub fn with_stop_on_ctrl_d(mut self, stop_on_ctrl_d: bool) -> Self {
        self.stop_on_ctrl_d = stop_on_ctrl_d;

        self
    }

    /// Turn on quick completions. These completions will auto-select if the completer
    /// ever narrows down to a single entry.
    pub fn with_quick_completions(mut self, quick_completions: bool) -> Self {
        self.quick_completions = quick_completions;

        self
    }

    /// Turn on partial completions. These completions will fill the buffer with the
    /// smallest common string from all the options
    pub fn with_partial_completions(mut self, partial_completions: bool) -> Self {
        self.partial_completions = partial_completions;

        self
    }

    /// Sets the style for reedline's fish-style history autosuggestions
    ///
    /// Default: `nu_ansi_term::Style::new().italic().fg(nu_ansi_term::Color::LightGray)`
    ///
    pub fn with_hinter_style(mut self, style: Style) -> Self {
        self.hinter_style = style;

        self
    }

    /// Disables reedline's fish-style history autosuggestions
    pub fn with_hinter_disabled(mut self) -> Self {
        self.hinter_enabled = false;

        self
    }

    /// Adds a reedline keybinding
    ///
    /// # Panics
    ///
    /// If `comamnd` is an empty [`ReedlineEvent::UntilFound`]
    pub fn with_keybinding(
        mut self,
        modifier: KeyModifiers,
        key_code: KeyCode,
        command: ReedlineEvent,
    ) -> Self {
        self.keybindings.add_binding(modifier, key_code, command);

        self
    }

    /// Find a keybinding based on the modifier and keycode
    pub fn find_keybinding(
        &self,
        modifier: KeyModifiers,
        key_code: KeyCode,
    ) -> Option<ReedlineEvent> {
        self.keybindings.find_binding(modifier, key_code)
    }

    /// Get assigned keybindings
    pub fn get_keybindings(&self) -> HashMap<(KeyModifiers, KeyCode), ReedlineEvent> {
        // keybindings.get_keybindings() cannot be returned directly because KeyCombination is not visible
        self.keybindings
            .get_keybindings()
            .iter()
            .map(|(key, value)| ((key.modifier, key.key_code), value.clone()))
            .collect()
    }

    /// Remove a keybinding
    ///
    /// Returns `Some(ReedlineEvent)` if the keycombination was previously bound to a particular [`ReedlineEvent`]
    pub fn without_keybinding(mut self, modifier: KeyModifiers, key_code: KeyCode) -> Self {
        self.keybindings.remove_binding(modifier, key_code);

        self
    }

    /// Access the external printer to print messages on the console
    ///
    /// The external printer can be used to print messages on the console without a command running.
    ///
    /// Example:
    ///
    /// ```rust,no_run
    /// use reedline_repl_rs::{Repl, Error};
    /// let mut repl: Repl<(), Error> = Repl::new(());
    /// let printer = repl.external_printer();
    /// std::thread::spawn(move || {
    ///     printer.print("hello world".to_owned()).expect("failed to print");
    /// });
    /// repl.run().expect("failed to run");
    /// ```
    pub fn external_printer(&self) -> ExternalPrinter<String> {
        self.external_printer.clone()
    }

    /// Add a command to your REPL
    pub fn with_command(mut self, command: Command, callback: Callback<Context, E>) -> Self {
        let name = command.get_name().to_string();
        self.commands
            .insert(name.clone(), ReplCommand::new(&name, command, callback));
        self
    }

    #[cfg(feature = "async_derive")]
    pub fn with_async_derived<Clap: clap::Parser>(
        mut self,
        callbacks: crate::AsyncCallBackMap<Context, E>,
    ) -> Self {
        let derived = Clap::command();

        self = self.with_name(derived.get_name());

        if let Some(version) = derived.get_version() {
            self = self.with_version(version);
        }

        if let Some(desc) = derived.get_about() {
            self = self.with_description(&desc.to_string());
        }

        let commands = derived.get_subcommands();

        for command in commands {
            let name = command.get_name();
            let Some(callback) = callbacks.get(name) else {
                continue;
            };
            let cmd: ReplCommand<Context, E> =
                ReplCommand::new_async(&name, command.clone(), *callback);

            self.commands.insert(name.to_string(), cmd);
        }

        self
    }

    /// Add a command to your REPL using clap's derive parser
    #[cfg(feature = "derive")]
    pub fn with_derived<Clap: clap::Parser>(
        mut self,
        callbacks: crate::CallBackMap<Context, E>,
    ) -> Self {
        let derived = Clap::command();

        self = self.with_name(derived.get_name());

        if let Some(version) = derived.get_version() {
            self = self.with_version(version);
        }

        if let Some(desc) = derived.get_about() {
            self = self.with_description(&desc.to_string());
        }

        let commands = derived.get_subcommands();

        for command in commands {
            let name = command.get_name();
            let Some(callback) = callbacks.get(name) else {
                continue;
            };
            let cmd: ReplCommand<Context, E> = ReplCommand::new(name, command.clone(), *callback);

            self.commands.insert(name.to_string(), cmd);
        }

        self
    }

    /// Add a command to your REPL
    #[cfg(feature = "async")]
    pub fn with_command_async(
        mut self,
        command: Command,
        callback: AsyncCallback<Context, E>,
    ) -> Self {
        let name = command.get_name().to_string();
        self.commands.insert(
            name.clone(),
            ReplCommand::new_async(&name, command, callback),
        );
        self
    }

    fn show_help(&self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            let mut app = Command::new("app");

            for (_, com) in self.commands.iter() {
                app = app.subcommand(com.command.clone());
            }
            let mut help_bytes: Vec<u8> = Vec::new();
            app.write_help(&mut help_bytes)
                .expect("failed to print help");
            let mut help_string =
                String::from_utf8(help_bytes).expect("Help message was invalid UTF8");
            let marker = "SUBCOMMANDS:";
            if let Some(marker_pos) = help_string.find(marker) {
                help_string = paint_yellow_bold("COMMANDS:")
                    + &help_string[(marker_pos + marker.len())..help_string.len()];
            }
            let header = format!(
                "{} {}\n{}\n",
                paint_green_bold(&self.name),
                self.version,
                self.description
            );
            println!("{}", header);
            println!("{}", help_string);
        } else if let Some((_, subcommand)) = self
            .commands
            .iter()
            .find(|(name, _)| name.as_str() == args[0])
        {
            subcommand
                .command
                .clone()
                .print_help()
                .expect("failed to print help");
            println!();
        } else {
            eprintln!("Help not found for command '{}'", args[0]);
        }
        Ok(())
    }

    fn handle_command(&mut self, command: &str, args: &[&str]) -> core::result::Result<(), E> {
        match self.commands.get(command) {
            Some(definition) => {
                let mut argv: Vec<&str> = vec![command];
                argv.extend(args);
                match definition.command.clone().try_get_matches_from_mut(argv) {
                    Ok(matches) => match (definition
                        .callback
                        .expect("Must be filled for sync commands"))(
                        matches, &mut self.context
                    ) {
                        Ok(Some(value)) => println!("{}", value),
                        Ok(None) => (),
                        Err(error) => return Err(error),
                    },
                    Err(err) => {
                        err.print().expect("failed to print");
                    }
                };
                self.execute_after_command_callback()?;
            }
            None => {
                if command == "help" {
                    self.show_help(args)?;
                } else {
                    return Err(Error::UnknownCommand(command.to_string()).into());
                }
            }
        }

        Ok(())
    }

    fn execute_after_command_callback(&mut self) -> core::result::Result<(), E> {
        if let Some(callback) = self.after_command_callback {
            match callback(&mut self.context) {
                Ok(Some(new_prompt)) => {
                    self.prompt.update_prefix(&new_prompt);
                }
                Ok(None) => {}
                Err(err) => {
                    eprintln!("failed to execute after_command_callback {:?}", err);
                }
            }
        }

        Ok(())
    }

    #[cfg(feature = "async")]
    async fn execute_after_command_callback_async(&mut self) -> core::result::Result<(), E> {
        self.execute_after_command_callback()?;
        if let Some(callback) = self.after_command_callback_async {
            match callback(&mut self.context).await {
                Ok(new_prompt) => {
                    if let Some(new_prompt) = new_prompt {
                        self.prompt.update_prefix(&new_prompt);
                    }
                }
                Err(err) => {
                    eprintln!("failed to execute after_command_callback {:?}", err);
                }
            }
        }

        Ok(())
    }

    #[cfg(feature = "async")]
    async fn handle_command_async(
        &mut self,
        command: &str,
        args: &[&str],
    ) -> core::result::Result<(), E> {
        match self.commands.get(command) {
            Some(definition) => {
                let mut argv: Vec<&str> = vec![command];
                argv.extend(args);
                match definition.command.clone().try_get_matches_from_mut(argv) {
                    Ok(matches) => match if let Some(async_callback) = definition.async_callback {
                        async_callback(matches, &mut self.context).await
                    } else {
                        definition
                            .callback
                            .expect("Either async or sync callback must be set")(
                            matches,
                            &mut self.context,
                        )
                    } {
                        Ok(Some(value)) => println!("{}", value),
                        Ok(None) => (),
                        Err(error) => return Err(error),
                    },
                    Err(err) => {
                        err.print().expect("failed to print");
                    }
                };
                self.execute_after_command_callback_async().await?;
            }
            None => {
                if command == "help" {
                    self.show_help(args)?;
                } else {
                    return Err(Error::UnknownCommand(command.to_string()).into());
                }
            }
        }

        Ok(())
    }

    #[cfg(not(feature = "shlex"))]
    fn parse_line(&self, line: &str) -> Option<Vec<String>> {
        let r = regex::Regex::new(r#"("[^"\n]+"|[\S]+)"#).unwrap();
        Some(r.captures_iter(line)
              .map(|a| a[0].to_string().replace('\"', ""))
              .collect::<Vec<String>>())
    }

    #[cfg(feature = "shlex")]
    fn parse_line(&self, line: &str) -> Option<Vec<String>> {
        shlex::split(line)
    }

    /// Process an array of arguments directly as command input.
    /// This method is called internally after splitting an input line.
    ///
    /// May be used instead of `run_with_reader` to execute a single command.
    /// An example use case is to process command line args directly as a command:
    ///
    /// ``` no_run
    /// //
    /// let mut repl = reedline_repl_rs::Repl::new(());
    /// // ... set up repl ...
    /// if std::env::args().len() > 1 {
    ///     repl.process_argv(std::env::args().skip(1).collect::<Vec<String>>())?;
    /// } else {
    ///     repl.run()?;
    /// }
    /// ```
    pub fn process_argv(&mut self, argv: Vec<String>) -> core::result::Result<(), E> {
        let mut iter = argv.iter();
        if let Some(command) = iter.next() {
            self.handle_command(command, &iter.map(AsRef::as_ref).collect::<Vec<&str>>())
        } else {
            Ok(())
        }
    }

    #[cfg(feature = "async")]
    /// Async version of `process_argv`, please refer to `process_argv` for documentation
    pub async fn process_argv_async(&mut self, argv: Vec<String>) -> core::result::Result<(), E> {
        let mut iter = argv.iter();
        if let Some(command) = iter.next() {
            self.handle_command_async(command, &iter.map(AsRef::as_ref).collect::<Vec<&str>>()).await
        } else {
            Ok(())
        }
    }

    fn process_line(&mut self, line: String) -> core::result::Result<(), E> {
        if let Some(args) = self.parse_line(&line) {
            self.process_argv(args)
        } else {
            Ok(())
        }
    }

    #[cfg(feature = "async")]
    async fn process_line_async(&mut self, line: String) -> core::result::Result<(), E> {
        if let Some(args) = self.parse_line(&line) {
            self.process_argv_async(args).await
        } else {
            Ok(())
        }
    }

    fn build_line_editor(&mut self) -> Result<Reedline> {
        let mut valid_commands: Vec<String> = self
            .commands
            .iter()
            .map(|(_, command)| command.name.clone())
            .collect();
        valid_commands.push("help".to_string());
        let completer = Box::new(ReplCompleter::new(&self.commands));
        let completion_menu = Box::new(ColumnarMenu::default().with_name("completion_menu"));
        let validator = Box::new(DefaultValidator);
        let mut line_editor = Reedline::create()
            .with_edit_mode(Box::new(Emacs::new(self.keybindings.clone())))
            .with_completer(completer)
            .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
            .with_highlighter(Box::new(ExampleHighlighter::new(valid_commands.clone())))
            .with_validator(validator)
            .with_partial_completions(self.partial_completions)
            .with_quick_completions(self.quick_completions)
            .with_external_printer(self.external_printer.clone());

        if self.hinter_enabled {
            line_editor = line_editor.with_hinter(Box::new(
                DefaultHinter::default().with_style(self.hinter_style),
            ));
        }

        if let Some(history_path) = &self.history {
            let capacity = self.history_capacity.unwrap();
            let history =
                FileBackedHistory::with_file(capacity, history_path.to_path_buf()).unwrap();
            line_editor = line_editor.with_history(Box::new(history));
        }

        Ok(line_editor)
    }

    #[cfg(feature = "scripts")]
    /// Executs REPL taking an object with a `std::io::BufRead` implementation
    /// as input
    /// This is useful for executing scripts. Exampel structure that can be used here
    /// is `std::io::BufReader` built on `std::fs::File`
    pub fn run_with_reader(&mut self, reader: impl std::io::BufRead) -> Result<()> {
        let lines = reader.lines();
        for line in lines {
            let line = line.expect("failed to read line");
            if let Err(err) = self.process_line(line) {
                (self.error_handler)(err, self)?;
            }
        }

        Ok(())
    }

    /// Execute REPL
    pub fn run(&mut self) -> Result<()> {
        self.enable_virtual_terminal_processing();
        if let Some(banner) = &self.banner {
            println!("{}", banner);
        }
        let mut line_editor = self.build_line_editor()?;

        loop {
            let sig = line_editor
                .read_line(&self.prompt)
                .expect("failed to read_line");
            match sig {
                Signal::Success(line) => {
                    if let Err(err) = self.process_line(line) {
                        (self.error_handler)(err, self)?;
                    }
                }
                Signal::CtrlC => {
                    if self.stop_on_ctrl_c {
                        break;
                    }
                }
                Signal::CtrlD => {
                    if self.stop_on_ctrl_d {
                        break;
                    }
                }
            }
        }
        self.restore_virtual_terminal_processing_mode();
        Ok(())
    }

    /// Execute REPL
    #[cfg(feature = "async")]
    pub async fn run_async(&mut self) -> Result<()> {
        self.enable_virtual_terminal_processing();
        if let Some(banner) = &self.banner {
            println!("{}", banner);
        }
        let mut line_editor = self.build_line_editor()?;

        loop {
            let sig = line_editor
                .read_line(&self.prompt)
                .expect("failed to read_line");
            match sig {
                Signal::Success(line) => {
                    if let Err(err) = self.process_line_async(line).await {
                        (self.error_handler)(err, self)?;
                    }
                }
                Signal::CtrlC => {
                    if self.stop_on_ctrl_c {
                        break;
                    }
                }
                Signal::CtrlD => {
                    if self.stop_on_ctrl_d {
                        break;
                    }
                }
            }
        }
        self.restore_virtual_terminal_processing_mode();
        Ok(())
    }

    fn enable_virtual_terminal_processing(&mut self) {
        #[cfg(windows)]
        {
            use winapi_util::console::{self, Console};
            self.restore_stdout_mode = console::mode(std::io::stdout()).ok();
            self.restore_stderr_mode = console::mode(std::io::stderr()).ok();
            if let Ok(mut term) = Console::stdout() {
                let _guard = term.set_virtual_terminal_processing(true);
            }
            if let Ok(mut term) = Console::stderr() {
                let _guard = term.set_virtual_terminal_processing(true);
            }
        }
    }

    fn restore_virtual_terminal_processing_mode(&mut self) {
        #[cfg(windows)]
        {
            use winapi_util::console;
            if let Some(mode) = self.restore_stdout_mode.take() {
                let _guard = console::set_mode(std::io::stdout(), mode);
            }
            if let Some(mode) = self.restore_stderr_mode.take() {
                let _guard = console::set_mode(std::io::stderr(), mode);
            }
        }
    }
}

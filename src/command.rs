#[cfg(feature = "async")]
use crate::AsyncCallback;
use crate::Callback;
use clap::Command;
use std::fmt;

/// Struct to define a command in the REPL
pub(crate) struct ReplCommand<Context, E> {
    pub(crate) name: String,
    pub(crate) command: Command,
    pub(crate) callback: Option<Callback<Context, E>>,
    #[cfg(feature = "async")]
    pub(crate) async_callback: Option<AsyncCallback<Context, E>>,
}

impl<Context, E> fmt::Debug for ReplCommand<Context, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Command").field("name", &self.name).finish()
    }
}

impl<Context, E> PartialEq for ReplCommand<Context, E> {
    fn eq(&self, other: &ReplCommand<Context, E>) -> bool {
        self.name == other.name
    }
}

impl<Context, E> ReplCommand<Context, E> {
    /// Create a new command with the given name and callback function
    pub fn new(name: &str, command: Command, callback: Callback<Context, E>) -> Self {
        Self {
            name: name.to_string(),
            command,
            callback: Some(callback),
            #[cfg(feature = "async")]
            async_callback: None,
        }
    }

    /// Create a new async command with the given name and callback function
    #[cfg(feature = "async")]
    pub fn new_async(name: &str, command: Command, callback: AsyncCallback<Context, E>) -> Self {
        Self {
            name: name.to_string(),
            command,
            callback: None,
            async_callback: Some(callback),
        }
    }
}

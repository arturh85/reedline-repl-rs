use reedline::{DefaultPrompt, Prompt, PromptEditMode, PromptHistorySearch};
use std::borrow::Cow;

#[derive(Clone)]
pub struct ReplPrompt {
    default: DefaultPrompt,
    prefix: String,
}

impl Prompt for ReplPrompt {
    /// Use prefix as render prompt
    fn render_prompt_left(&self) -> Cow<'_, str> {
        {
            Cow::Borrowed(&self.prefix)
        }
    }

    // call default impl
    fn render_prompt_right(&self) -> Cow<'_, str> {
        self.default.render_prompt_right()
    }
    fn render_prompt_indicator(&self, edit_mode: PromptEditMode) -> Cow<'_, str> {
        self.default.render_prompt_indicator(edit_mode)
    }
    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        self.default.render_prompt_multiline_indicator()
    }
    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<'_, str> {
        self.default
            .render_prompt_history_search_indicator(history_search)
    }
}

impl Default for ReplPrompt {
    fn default() -> Self {
        ReplPrompt::new("repl")
    }
}

impl ReplPrompt {
    /// Constructor for the default prompt, which takes the amount of spaces required between the left and right-hand sides of the prompt
    pub fn new(left_prompt: &str) -> ReplPrompt {
        ReplPrompt {
            prefix: left_prompt.to_string(),
            default: DefaultPrompt::default(),
        }
    }

    #[allow(dead_code)]
    pub fn update_prefix(&mut self, prefix: &str) {
        self.prefix = prefix.to_string();
    }
}

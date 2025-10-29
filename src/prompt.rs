use reedline::{DefaultPrompt, DefaultPromptSegment, Prompt, PromptEditMode, PromptHistorySearch};
use std::borrow::Cow;

#[derive(Clone)]
pub struct ReplPrompt {
    default: DefaultPrompt,
}

impl Prompt for ReplPrompt {
    // call default impl
    fn render_prompt_left(&self) -> Cow<str> {
        self.default.render_prompt_left()
    }
    fn render_prompt_right(&self) -> Cow<str> {
        self.default.render_prompt_right()
    }
    fn render_prompt_indicator(&self, edit_mode: PromptEditMode) -> Cow<str> {
        self.default.render_prompt_indicator(edit_mode)
    }
    fn render_prompt_multiline_indicator(&self) -> Cow<str> {
        self.default.render_prompt_multiline_indicator()
    }
    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<str> {
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
        let mut prompt = ReplPrompt {
            default: DefaultPrompt::default(),
        };
        prompt.update_prefix(left_prompt);
        prompt
    }

    pub fn update_prefix(&mut self, prefix: &str) {
        self.default.left_prompt = DefaultPromptSegment::Basic(prefix.to_string());
    }

    pub fn disable_clock(&mut self) {
        self.default.right_prompt = DefaultPromptSegment::Empty;
    }
}

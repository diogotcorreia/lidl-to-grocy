use inquire::{error::InquireResult, Confirm, DateSelect, MultiSelect, Select, Text};

// Grocy-style...
fn bool_to_string(val: bool) -> String {
    if val {
        "1".to_owned()
    } else {
        "0".to_owned()
    }
}

fn vec_to_string(vec: Vec<String>) -> String {
    vec.join(",")
}

// needed to make use of generics since `dyn Prompt` does not allow
// calling the trait functions (they take self instead of &self); this
// essentially just dispatches to one of those methods
pub fn prompt<T: Prompt>(prompt: T, skippable: bool) -> InquireResult<Option<String>> {
    if skippable {
        Ok(prompt.prompt_skippable()?)
    } else {
        Ok(Some(prompt.prompt()?))
    }
}

pub trait Prompt {
    fn prompt(self) -> InquireResult<String>;
    fn prompt_skippable(self) -> InquireResult<Option<String>>;
}

impl Prompt for Confirm<'_> {
    fn prompt(self) -> InquireResult<String> {
        self.prompt().map(bool_to_string)
    }

    fn prompt_skippable(self) -> InquireResult<Option<String>> {
        self.prompt_skippable().map(|op| op.map(bool_to_string))
    }
}

impl Prompt for Text<'_> {
    fn prompt(self) -> InquireResult<String> {
        self.prompt()
    }

    fn prompt_skippable(self) -> InquireResult<Option<String>> {
        self.prompt_skippable()
    }
}

impl Prompt for DateSelect<'_> {
    fn prompt(self) -> InquireResult<String> {
        self.prompt().map(|date| date.to_string())
    }

    fn prompt_skippable(self) -> InquireResult<Option<String>> {
        self.prompt_skippable()
            .map(|op| op.map(|date| date.to_string()))
    }
}

impl Prompt for Select<'_, String> {
    fn prompt(self) -> InquireResult<String> {
        self.prompt()
    }

    fn prompt_skippable(self) -> InquireResult<Option<String>> {
        self.prompt_skippable()
    }
}

impl Prompt for MultiSelect<'_, String> {
    fn prompt(self) -> InquireResult<String> {
        self.prompt().map(vec_to_string)
    }

    fn prompt_skippable(self) -> InquireResult<Option<String>> {
        self.prompt_skippable().map(|op| op.map(vec_to_string))
    }
}

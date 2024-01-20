use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Country {
    pub(crate) id: String,
    default_name: String,
    active: bool,
    languages: Vec<Language>,
}

impl Country {
    pub fn get_default_language(&self) -> Option<Language> {
        self.languages
            .iter()
            .find(|language| language.active)
            .cloned()
    }
}

impl Display for Country {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.id, self.default_name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Language {
    pub(crate) id: String,
    default_name: String,
    active: bool,
    default: bool,
}

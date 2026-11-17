use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Doc {
    pub title: String,
    pub date: String,
    pub body: Arc<str>,
    #[serde(default)]
    pub font: usize,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Shelf {
    pub docs: Vec<Doc>,
    pub active: usize,
    pub last: usize,
}

impl Default for Shelf {
    fn default() -> Self {
        Self {
            docs: serde_json::from_str(include_str!("../assets/welcome.json"))
                .expect("embedded welcome documents"),
            active: 0,
            last: 0,
        }
    }
}

impl Shelf {
    pub(crate) fn validate(&self) -> Result<(), &'static str> {
        if self.docs.is_empty() || self.active >= self.docs.len() {
            return Err("invalid active document");
        }
        if self.last >= 4 || self.docs.iter().any(|d| d.font >= 4) {
            return Err("invalid font selection");
        }
        Ok(())
    }
}

use std::ops::{Deref, DerefMut};

use url::Url;
use diff::Diff;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct DiffUrl(Url);

impl Deref for DiffUrl {
    type Target = Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DiffUrl {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Diff for DiffUrl {
    type Repr = Option<Url>;

    fn diff(&self, other: &Self) -> Self::Repr {
        if self.0 == other.0 {None} else {Some(other.0.clone())}
    }

    fn apply(&mut self, diff: &Self::Repr) {
        if let Some(diff) = diff {
			self.0 = diff.clone()
		}
    }

    fn identity() -> Self {
        Self(Url::parse("https://example.com").unwrap())
    }
}
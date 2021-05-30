use crate::StrictEq;
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap};

#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CodeBlock<'a> {
    pub lang: Option<Cow<'a, str>>,
    pub metadata: HashMap<Cow<'a, str>, Cow<'a, str>>,
    pub lines: Vec<Cow<'a, str>>,
}

impl<'a> CodeBlock<'a> {
    /// Constructs a code block with the provided lines using no language or metadata
    pub fn from_lines<I: IntoIterator<Item = L>, L: Into<Cow<'a, str>>>(
        iter: I,
    ) -> Self {
        Self {
            lang: None,
            metadata: HashMap::new(),
            lines: iter.into_iter().map(Into::into).collect(),
        }
    }
}

impl CodeBlock<'_> {
    pub fn to_borrowed(&self) -> CodeBlock {
        use self::Cow::*;

        CodeBlock {
            lang: self.lang.as_ref().map(|x| {
                Cow::Borrowed(match x {
                    Borrowed(x) => *x,
                    Owned(x) => x.as_str(),
                })
            }),
            metadata: self
                .metadata
                .iter()
                .map(|(key, value)| {
                    let key = Cow::Borrowed(match key {
                        Borrowed(x) => *x,
                        Owned(x) => x.as_str(),
                    });
                    let value = Cow::Borrowed(match value {
                        Borrowed(x) => *x,
                        Owned(x) => x.as_str(),
                    });

                    (key, value)
                })
                .collect(),
            lines: self
                .lines
                .iter()
                .map(|x| {
                    Cow::Borrowed(match x {
                        Borrowed(x) => *x,
                        Owned(x) => x.as_str(),
                    })
                })
                .collect(),
        }
    }

    pub fn into_owned(self) -> CodeBlock<'static> {
        CodeBlock {
            lang: self.lang.map(|x| Cow::from(x.into_owned())),
            metadata: self
                .metadata
                .into_iter()
                .map(|(key, value)| {
                    (Cow::from(key.into_owned()), Cow::from(value.into_owned()))
                })
                .collect(),
            lines: self
                .lines
                .into_iter()
                .map(|x| Cow::from(x.into_owned()))
                .collect(),
        }
    }
}

impl<'a> StrictEq for CodeBlock<'a> {
    /// Same as PartialEq
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}
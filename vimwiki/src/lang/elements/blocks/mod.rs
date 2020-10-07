use super::LE;
use derive_more::From;
use serde::{Deserialize, Serialize};

mod blockquotes;
pub use blockquotes::*;
mod definitions;
pub use definitions::*;
mod dividers;
pub use dividers::*;
mod headers;
pub use headers::*;
mod inline;
pub use inline::*;
mod lists;
pub use lists::*;
mod math;
pub use math::*;
mod paragraphs;
pub use paragraphs::*;
mod placeholders;
pub use placeholders::*;
mod preformatted;
pub use preformatted::*;
mod tables;
pub use tables::*;

/// Represents elements that are standalone (metaphorically a block element in CSS)
#[derive(Clone, Debug, From, Eq, PartialEq, Serialize, Deserialize)]
pub enum BlockElement {
    BlankLine,
    Blockquote(Blockquote),
    DefinitionList(DefinitionList),
    Divider(Divider),
    Header(Header),
    List(List),
    Math(MathBlock),
    NonBlankLine(String),
    Paragraph(Paragraph),
    Placeholder(Placeholder),
    PreformattedText(PreformattedText),
    Table(Table),
}

macro_rules! lc_mapping {
    ($type:ty) => {
        impl From<LE<$type>> for LE<BlockElement> {
            fn from(element: LE<$type>) -> Self {
                element.map(BlockElement::from)
            }
        }
    };
}

lc_mapping!(Header);
lc_mapping!(Paragraph);
lc_mapping!(DefinitionList);
lc_mapping!(List);
lc_mapping!(Table);
lc_mapping!(PreformattedText);
lc_mapping!(MathBlock);
lc_mapping!(Blockquote);
lc_mapping!(Divider);
lc_mapping!(Placeholder);
lc_mapping!(String);
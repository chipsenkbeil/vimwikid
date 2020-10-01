use super::{
    components::{self, BlockComponent},
    utils::{self, context, lc, VimwikiIResult},
    Span, LC,
};
use nom::{
    branch::alt,
    combinator::{map, value},
};

pub mod blockquotes;
pub mod definitions;
pub mod dividers;
pub mod headers;
pub mod inline;
pub mod lists;
pub mod math;
pub mod paragraphs;
pub mod placeholders;
pub mod preformatted;
pub mod tables;

/// Parses a block component
pub fn block_component(input: Span) -> VimwikiIResult<LC<BlockComponent>> {
    context(
        "Block Component",
        alt((
            map(headers::header, |c| c.map(BlockComponent::from)),
            map(definitions::definition_list, |c| {
                c.map(BlockComponent::from)
            }),
            map(lists::list, |c| c.map(BlockComponent::from)),
            map(tables::table, |c| c.map(BlockComponent::from)),
            map(preformatted::preformatted_text, |c| {
                c.map(BlockComponent::from)
            }),
            map(math::math_block, |c| c.map(BlockComponent::from)),
            map(blank_line, |c| LC::new(BlockComponent::BlankLine, c.region)),
            map(blockquotes::blockquote, |c| c.map(BlockComponent::from)),
            map(dividers::divider, |c| c.map(BlockComponent::from)),
            map(placeholders::placeholder, |c| c.map(BlockComponent::from)),
            map(paragraphs::paragraph, |c| c.map(BlockComponent::from)),
            // NOTE: Parses a single line to end; final type because will match
            //       anychar and consume the line; used as our fallback in
            //       case we don't match any other type
            map(non_blank_line, |c| c.map(BlockComponent::from)),
        )),
    )(input)
}

/// Parses a blank line
fn blank_line(input: Span) -> VimwikiIResult<LC<()>> {
    context("Blank Line", lc(value((), utils::blank_line)))(input)
}

/// Parses a non-blank line
fn non_blank_line(input: Span) -> VimwikiIResult<LC<String>> {
    context("Non Blank Line", lc(utils::non_blank_line))(input)
}
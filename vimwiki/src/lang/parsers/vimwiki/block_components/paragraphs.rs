use super::{
    blockquotes::blockquote,
    components::Paragraph,
    definitions::definition_list,
    dividers::divider,
    headers::header,
    inline::inline_component_container,
    lists::list,
    math::math_block,
    placeholders::placeholder,
    preformatted::preformatted_text,
    tables::table,
    utils::{beginning_of_line, blank_line, context, end_of_line_or_input, lc},
    Span, VimwikiIResult, LC,
};
use nom::{
    character::complete::space1,
    combinator::{map, not},
    multi::many1,
    sequence::delimited,
};

/// Parses a vimwiki paragraph, returning the associated paragraph is successful
#[inline]
pub fn paragraph(input: Span) -> VimwikiIResult<LC<Paragraph>> {
    fn inner(input: Span) -> VimwikiIResult<Paragraph> {
        // Ensure that we are starting at the beginning of a line
        let (input, _) = beginning_of_line(input)?;

        // Paragraph has NO indentation
        let (input, _) = not(space1)(input)?;

        // Continuously take content until we encounter another type of
        // component
        let (input, components) = context(
            "Paragraph",
            many1(delimited(
                continue_paragraph,
                map(inline_component_container, |c| c.component),
                end_of_line_or_input,
            )),
        )(input)?;

        // Transform contents into the paragraph itself
        let paragraph = Paragraph::new(From::from(components));

        Ok((input, paragraph))
    }

    context("Paragraph", lc(inner))(input)
}

// TODO: Optimize by adjusting paragraph parser to be a tuple that
//       includes an Option<BlockComponent> so that we don't waste
//       the processing spent
fn continue_paragraph(input: Span) -> VimwikiIResult<()> {
    let (input, _) = not(header)(input)?;
    let (input, _) = not(definition_list)(input)?;
    let (input, _) = not(list)(input)?;
    let (input, _) = not(table)(input)?;
    let (input, _) = not(preformatted_text)(input)?;
    let (input, _) = not(math_block)(input)?;
    let (input, _) = not(blank_line)(input)?;
    let (input, _) = not(blockquote)(input)?;
    let (input, _) = not(divider)(input)?;
    let (input, _) = not(placeholder)(input)?;
    Ok((input, ()))
}

#[cfg(test)]
mod tests {
    use super::super::components::{
        DecoratedText, DecoratedTextContent, Decoration, InlineComponent, Link,
        MathInline, WikiLink,
    };
    use super::*;
    use crate::lang::utils::Span;
    use indoc::indoc;
    use std::path::PathBuf;

    #[test]
    fn paragraph_should_fail_if_on_blank_line() {
        let input = Span::from(" ");
        assert!(paragraph(input).is_err());
    }

    #[test]
    fn paragraph_should_fail_if_line_indented() {
        let input = Span::from(" some text");
        assert!(paragraph(input).is_err());
    }

    #[test]
    fn paragraph_should_parse_single_line() {
        let input = Span::from(indoc! {"
        Some paragraph with *decorations*, [[links]], $math$, and more
        "});
        let (input, mut p) = paragraph(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume paragraph");

        assert_eq!(
            p.content
                .components
                .drain(..)
                .map(|c| c.component)
                .collect::<Vec<InlineComponent>>(),
            vec![
                InlineComponent::Text("Some paragraph with ".to_string()),
                InlineComponent::DecoratedText(DecoratedText::new(
                    vec![LC::from(DecoratedTextContent::Text(
                        "decorations".to_string()
                    ))],
                    Decoration::Bold
                )),
                InlineComponent::Text(", ".to_string()),
                InlineComponent::Link(Link::from(WikiLink::from(
                    PathBuf::from("links")
                ))),
                InlineComponent::Text(", ".to_string()),
                InlineComponent::Math(MathInline::new("math".to_string())),
                InlineComponent::Text(", and more".to_string()),
            ],
        );
    }

    #[test]
    fn paragraph_should_parse_multiple_lines() {
        let input = Span::from(indoc! {"
        Some paragraph with *decorations*,
        [[links]], $math$, and more
        "});
        let (input, mut p) = paragraph(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume paragraph");

        assert_eq!(
            p.content
                .components
                .drain(..)
                .map(|c| c.component)
                .collect::<Vec<InlineComponent>>(),
            vec![
                InlineComponent::Text("Some paragraph with ".to_string()),
                InlineComponent::DecoratedText(DecoratedText::new(
                    vec![LC::from(DecoratedTextContent::Text(
                        "decorations".to_string()
                    ))],
                    Decoration::Bold
                )),
                InlineComponent::Text(",".to_string()),
                InlineComponent::Link(Link::from(WikiLink::from(
                    PathBuf::from("links")
                ))),
                InlineComponent::Text(", ".to_string()),
                InlineComponent::Math(MathInline::new("math".to_string())),
                InlineComponent::Text(", and more".to_string()),
            ],
        );
    }

    #[test]
    fn paragraph_should_support_whitespace_at_beginning_of_all_following_lines()
    {
        let input = Span::from(indoc! {"
        Some paragraph with *decorations*,
          [[links]], $math$, and more
        "});
        let (input, mut p) = paragraph(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume paragraph");

        assert_eq!(
            p.content
                .components
                .drain(..)
                .map(|c| c.component)
                .collect::<Vec<InlineComponent>>(),
            vec![
                InlineComponent::Text("Some paragraph with ".to_string()),
                InlineComponent::DecoratedText(DecoratedText::new(
                    vec![LC::from(DecoratedTextContent::Text(
                        "decorations".to_string()
                    ))],
                    Decoration::Bold
                )),
                InlineComponent::Text(",".to_string()),
                InlineComponent::Text("  ".to_string()),
                InlineComponent::Link(Link::from(WikiLink::from(
                    PathBuf::from("links")
                ))),
                InlineComponent::Text(", ".to_string()),
                InlineComponent::Math(MathInline::new("math".to_string())),
                InlineComponent::Text(", and more".to_string()),
            ],
        );
    }

    #[test]
    fn paragraph_should_stop_at_a_blank_line() {
        let input = Span::from(indoc! {"
        Some paragraph with *decorations*,
        [[links]], $math$, and more

        And this would be a second paragraph
        "});
        let (input, mut p) = paragraph(input).unwrap();
        assert_eq!(
            input.fragment_str(),
            "\nAnd this would be a second paragraph\n",
            "Unexpected consumption of input"
        );

        assert_eq!(
            p.content
                .components
                .drain(..)
                .map(|c| c.component)
                .collect::<Vec<InlineComponent>>(),
            vec![
                InlineComponent::Text("Some paragraph with ".to_string()),
                InlineComponent::DecoratedText(DecoratedText::new(
                    vec![LC::from(DecoratedTextContent::Text(
                        "decorations".to_string()
                    ))],
                    Decoration::Bold
                )),
                InlineComponent::Text(",".to_string()),
                InlineComponent::Link(Link::from(WikiLink::from(
                    PathBuf::from("links")
                ))),
                InlineComponent::Text(", ".to_string()),
                InlineComponent::Math(MathInline::new("math".to_string())),
                InlineComponent::Text(", and more".to_string()),
            ],
        );
    }
}
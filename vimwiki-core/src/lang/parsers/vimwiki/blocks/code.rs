use crate::lang::{
    elements::{CodeBlock, Located},
    parsers::{
        utils::{
            any_line, beginning_of_line, capture, context, cow_str,
            end_of_line_or_input, locate, take_line_until, take_line_until1,
        },
        IResult, Span,
    },
};
use nom::{
    bytes::complete::tag,
    character::complete::{char, space0, space1},
    combinator::{map_parser, not, opt, verify},
    multi::{many0, separated_list0},
    sequence::{delimited, preceded, separated_pair},
};
use std::{borrow::Cow, collections::HashMap};

type MaybeLang<'a> = Option<Cow<'a, str>>;
type Metadata<'a> = HashMap<Cow<'a, str>, Cow<'a, str>>;

#[inline]
pub fn code_block(input: Span) -> IResult<Located<CodeBlock>> {
    fn inner(input: Span) -> IResult<CodeBlock> {
        let (input, (maybe_lang, metadata)) = code_block_start(input)?;
        let (input, lines) = many0(preceded(
            not(code_block_end),
            map_parser(any_line, cow_str),
        ))(input)?;
        let (input, _) = code_block_end(input)?;

        Ok((input, CodeBlock::new(maybe_lang, metadata, lines)))
    }

    context("Preformatted Text", locate(capture(inner)))(input)
}

#[inline]
fn code_block_start<'a>(
    input: Span<'a>,
) -> IResult<(MaybeLang<'a>, Metadata<'a>)> {
    // First, verify we have the start of a block and consume it
    let (input, _) = beginning_of_line(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = tag("{{{")(input)?;

    // Second, look for optional language and consume it
    //
    // e.g. {{{c++ -> Some("c++")
    let (input, maybe_lang) = opt(map_parser(
        verify(take_line_until1(" "), |s: &Span| {
            !s.as_remaining().contains(&b'=')
        }),
        cow_str,
    ))(input)?;

    // Third, remove any extra spaces before metadata
    let (input, _) = space0(input)?;

    // Fourth, look for optional metadata and consume it
    //
    // e.g. {{{key1="value 1" key2="value 2"
    let (input, pairs) = separated_list0(
        space1,
        separated_pair(
            map_parser(take_line_until1("="), cow_str),
            char('='),
            delimited(
                char('"'),
                map_parser(take_line_until("\""), cow_str),
                char('"'),
            ),
        ),
    )(input)?;

    // Fifth, consume end of line
    let (input, _) = space0(input)?;
    let (input, _) = end_of_line_or_input(input)?;

    Ok((input, (maybe_lang, pairs.into_iter().collect())))
}

#[inline]
fn code_block_end(input: Span) -> IResult<()> {
    let (input, _) = beginning_of_line(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = tag("}}}")(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = end_of_line_or_input(input)?;

    Ok((input, ()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn code_block_should_fail_if_does_not_have_starting_line() {
        let input = Span::from(indoc! {r"
            some code
            }}}
        "});
        assert!(code_block(input).is_err());
    }

    #[test]
    fn code_block_should_fail_if_starting_block_not_on_own_line() {
        let input = Span::from(indoc! {r"
            {{{some code
            }}}
        "});
        assert!(code_block(input).is_err());
    }

    #[test]
    fn code_block_should_fail_if_does_not_have_ending_line() {
        let input = Span::from(indoc! {r"
            {{{
            some code
        "});
        assert!(code_block(input).is_err());
    }

    #[test]
    fn code_block_should_fail_if_ending_block_not_on_own_line() {
        let input = Span::from(indoc! {r"
            {{{
            some code}}}
        "});
        assert!(code_block(input).is_err());
    }

    #[test]
    fn code_block_should_support_having_no_lines() {
        let input = Span::from(indoc! {r"
            {{{
            }}}
        "});
        let (input, p) = code_block(input).unwrap();
        assert!(input.is_empty(), "Did not consume code block");
        assert!(p.language.is_none(), "Has unexpected language");
        assert!(p.lines.is_empty(), "Has unexpected lines");
        assert!(p.metadata.is_empty(), "Has unexpected metadata");
    }

    #[test]
    fn code_block_should_support_lang_shorthand() {
        let input = Span::from(indoc! {r"
            {{{c++
            some code
            }}}
        "});
        let (input, p) = code_block(input).unwrap();
        assert!(input.is_empty(), "Did not consume code block");
        assert_eq!(p.language.as_deref(), Some("c++"));
        assert_eq!(
            p.lines.iter().map(AsRef::as_ref).collect::<Vec<&str>>(),
            vec!["some code"]
        );
        assert!(p.metadata.is_empty(), "Has unexpected metadata");
    }

    #[test]
    fn code_block_should_support_lang_shorthand_with_metadata() {
        let input = Span::from(indoc! {r#"
            {{{c++ key="value"
            some code
            }}}
        "#});
        let (input, p) = code_block(input).unwrap();
        assert!(input.is_empty(), "Did not consume code block");
        assert_eq!(p.language.as_deref(), Some("c++"));
        assert_eq!(
            p.lines.iter().map(AsRef::as_ref).collect::<Vec<&str>>(),
            vec!["some code"]
        );
        assert_eq!(p.metadata.get("key"), Some(&Cow::from("value")));
    }

    #[test]
    fn code_block_should_parse_all_lines_between() {
        let input = Span::from(indoc! {r"
            {{{
            Tyger! Tyger! burning bright
             In the forests of the night,
              What immortal hand or eye
               Could frame thy fearful symmetry?
            In what distant deeps or skies
             Burnt the fire of thine eyes?
              On what wings dare he aspire?
               What the hand dare sieze the fire?
            }}}
        "});
        let (input, p) = code_block(input).unwrap();
        assert!(input.is_empty(), "Did not consume code block");
        assert_eq!(
            p.lines.iter().map(AsRef::as_ref).collect::<Vec<&str>>(),
            vec![
                "Tyger! Tyger! burning bright",
                " In the forests of the night,",
                "  What immortal hand or eye",
                "   Could frame thy fearful symmetry?",
                "In what distant deeps or skies",
                " Burnt the fire of thine eyes?",
                "  On what wings dare he aspire?",
                "   What the hand dare sieze the fire?",
            ]
        );
        assert!(p.language.is_none(), "Has unexpected language");
        assert!(p.metadata.is_empty(), "Has unexpected metadata");
    }

    #[test]
    fn code_block_should_support_single_metadata() {
        let input = Span::from(indoc! {r#"
            {{{class="brush: python"
            def hello(world):
                for x in range(10):
                    print("Hello {0} number {1}".format(world, x))
            }}}
        "#});
        let (input, p) = code_block(input).unwrap();
        assert!(input.is_empty(), "Did not consume code block");
        assert_eq!(
            p.lines.iter().map(AsRef::as_ref).collect::<Vec<&str>>(),
            vec![
                r#"def hello(world):"#,
                r#"    for x in range(10):"#,
                r#"        print("Hello {0} number {1}".format(world, x))"#,
            ]
        );
        assert_eq!(p.metadata.get("class"), Some(&Cow::from("brush: python")));
    }

    #[test]
    fn code_block_should_support_multiple_metadata() {
        let input = Span::from(indoc! {r#"
            {{{class="brush: python" style="position: relative"
            def hello(world):
                for x in range(10):
                    print("Hello {0} number {1}".format(world, x))
            }}}
        "#});
        let (input, p) = code_block(input).unwrap();
        assert!(input.is_empty(), "Did not consume code block");
        assert_eq!(
            p.lines.iter().map(AsRef::as_ref).collect::<Vec<&str>>(),
            vec![
                r#"def hello(world):"#,
                r#"    for x in range(10):"#,
                r#"        print("Hello {0} number {1}".format(world, x))"#,
            ]
        );
        assert!(p.language.is_none(), "Has unexpected language");
        assert_eq!(p.metadata.get("class"), Some(&Cow::from("brush: python")));
        assert_eq!(
            p.metadata.get("style"),
            Some(&Cow::from("position: relative"))
        );
    }
}

use super::{
    elements::Placeholder,
    utils::{
        beginning_of_line, context, end_of_line_or_input, lc, pstring,
        take_line_while1, take_until_end_of_line_or_input,
    },
    Span, VimwikiIResult, LC,
};
use chrono::NaiveDate;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{space0, space1},
    combinator::{map_res, not, verify},
};

#[inline]
pub fn placeholder(input: Span) -> VimwikiIResult<LC<Placeholder>> {
    fn inner(input: Span) -> VimwikiIResult<LC<Placeholder>> {
        let (input, _) = beginning_of_line(input)?;
        let (input, lc_placeholder) = lc(alt((
            placeholder_title,
            placeholder_nohtml,
            placeholder_template,
            placeholder_date,
            placeholder_other,
        )))(input)?;
        let (input, _) = end_of_line_or_input(input)?;
        Ok((input, lc_placeholder))
    }

    context("Placeholder", inner)(input)
}

fn placeholder_title(input: Span) -> VimwikiIResult<Placeholder> {
    fn inner(input: Span) -> VimwikiIResult<Placeholder> {
        let (input, _) = tag("%title")(input)?;
        let (input, _) = space1(input)?;
        let (input, text) =
            pstring(verify(take_until_end_of_line_or_input, |s: &Span| {
                !s.fragment_str().trim().is_empty()
            }))(input)?;
        Ok((input, Placeholder::Title(text)))
    }

    context("Placeholder Title", inner)(input)
}

fn placeholder_nohtml(input: Span) -> VimwikiIResult<Placeholder> {
    fn inner(input: Span) -> VimwikiIResult<Placeholder> {
        let (input, _) = tag("%nohtml")(input)?;
        let (input, _) = space0(input)?;
        Ok((input, Placeholder::NoHtml))
    }

    context("Placeholder NoHtml", inner)(input)
}

fn placeholder_template(input: Span) -> VimwikiIResult<Placeholder> {
    fn inner(input: Span) -> VimwikiIResult<Placeholder> {
        let (input, _) = tag("%template")(input)?;
        let (input, _) = space1(input)?;
        let (input, text) =
            pstring(verify(take_until_end_of_line_or_input, |s: &Span| {
                !s.fragment_str().trim().is_empty()
            }))(input)?;
        Ok((input, Placeholder::Template(text)))
    }

    context("Placeholder Template", inner)(input)
}

fn placeholder_date(input: Span) -> VimwikiIResult<Placeholder> {
    fn inner(input: Span) -> VimwikiIResult<Placeholder> {
        let (input, _) = tag("%date")(input)?;
        let (input, _) = space1(input)?;
        let (input, date) =
            map_res(take_until_end_of_line_or_input, |s: Span| {
                NaiveDate::parse_from_str(s.fragment_str(), "%Y-%m-%d")
            })(input)?;
        Ok((input, Placeholder::Date(date)))
    }

    context("Placeholder Date", inner)(input)
}

fn placeholder_other(input: Span) -> VimwikiIResult<Placeholder> {
    fn inner(input: Span) -> VimwikiIResult<Placeholder> {
        let (input, _) = not(tag("%title"))(input)?;
        let (input, _) = not(tag("%nohtml"))(input)?;
        let (input, _) = not(tag("%template"))(input)?;
        let (input, _) = not(tag("%date"))(input)?;

        let (input, _) = tag("%")(input)?;
        let (input, name) = pstring(take_line_while1(not(alt((
            tag(" "),
            tag("\t"),
            tag("%"),
        )))))(input)?;
        let (input, _) = space1(input)?;
        let (input, value) =
            pstring(verify(take_until_end_of_line_or_input, |s: &Span| {
                !s.fragment_str().trim().is_empty()
            }))(input)?;
        Ok((input, Placeholder::Other { name, value }))
    }

    context("Placeholder Other", inner)(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::utils::Span;

    #[test]
    fn placeholder_should_fail_if_input_empty() {
        let input = Span::from("");
        assert!(placeholder(input).is_err());
    }

    #[test]
    fn placeholder_should_fail_title_with_no_text() {
        let input = Span::from("%title");
        assert!(placeholder(input).is_err());
    }

    #[test]
    fn placeholder_should_succeed_if_title_with_text_input() {
        let input = Span::from("%title some title");
        let (input, placeholder) = placeholder(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume placeholder");
        assert_eq!(placeholder, Placeholder::Title("some title".to_string()));
    }

    #[test]
    fn placeholder_should_fail_if_nohtml_with_text() {
        let input = Span::from("%nohtml something");
        assert!(placeholder(input).is_err());
    }

    #[test]
    fn placeholder_should_succeed_if_nohtml_with_no_text_input() {
        let input = Span::from("%nohtml");
        let (input, placeholder) = placeholder(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume placeholder");
        assert_eq!(placeholder, Placeholder::NoHtml);
    }

    #[test]
    fn placeholder_should_fail_if_template_with_no_text() {
        let input = Span::from("%template");
        assert!(placeholder(input).is_err());
    }

    #[test]
    fn placeholder_should_succeed_if_template_with_text_input() {
        let input = Span::from("%template my_template");
        let (input, placeholder) = placeholder(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume placeholder");
        assert_eq!(
            placeholder,
            Placeholder::Template("my_template".to_string()),
        );
    }

    #[test]
    fn placeholder_should_fail_if_date_with_no_text() {
        let input = Span::from("%date");
        assert!(placeholder(input).is_err());
    }

    #[test]
    fn placeholder_should_fail_if_date_with_non_date_input() {
        let input = Span::from("%date something");
        assert!(placeholder(input).is_err());
    }

    #[test]
    fn placeholder_should_succeed_if_date_with_date_input() {
        let input = Span::from("%date 2012-03-05");
        let (input, placeholder) = placeholder(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume placeholder");
        assert_eq!(
            placeholder,
            Placeholder::Date(NaiveDate::from_ymd(2012, 3, 5)),
        );
    }

    #[test]
    fn placeholder_fallback_should_fail_if_double_percent_at_start() {
        let input = Span::from("%%other something else");
        assert!(placeholder(input).is_err());
    }

    #[test]
    fn placeholder_fallback_should_fail_if_no_space_between_name_and_value() {
        let input = Span::from("%othervalue");
        assert!(placeholder(input).is_err());
    }

    #[test]
    fn placeholder_fallback_should_fail_if_no_name_provided() {
        let input = Span::from("% value");
        assert!(placeholder(input).is_err());
    }

    #[test]
    fn placeholder_fallback_should_fail_if_percent_found_in_name() {
        let input = Span::from("%oth%er value");
        assert!(placeholder(input).is_err());
    }

    #[test]
    fn placeholder_fallback_should_fail_if_percent_found_at_end_of_name() {
        let input = Span::from("%other% value");
        assert!(placeholder(input).is_err());
    }

    #[test]
    fn placeholder_fallback_should_fail_if_no_value_after_name() {
        let input = Span::from("%other");
        assert!(placeholder(input).is_err());
    }

    #[test]
    fn placeholder_fallback_should_succeed_if_percent_followed_by_name_space_and_value(
    ) {
        let input = Span::from("%other something else");
        let (input, placeholder) = placeholder(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume placeholder");
        assert_eq!(
            placeholder,
            Placeholder::Other {
                name: "other".to_string(),
                value: "something else".to_string()
            },
        );
    }
}

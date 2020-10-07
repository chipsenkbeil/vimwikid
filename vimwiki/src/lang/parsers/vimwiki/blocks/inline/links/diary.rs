use super::{
    elements::DiaryLink,
    utils::{context, take_line_while1, VimwikiNomError},
    wiki::wiki_link,
    Span, VimwikiIResult, LE,
};
use chrono::NaiveDate;
use nom::{
    bytes::complete::tag, character::complete::anychar, sequence::preceded,
};

#[inline]
pub fn diary_link(input: Span) -> VimwikiIResult<LE<DiaryLink>> {
    fn inner(input: Span) -> VimwikiIResult<LE<DiaryLink>> {
        // First, parse as a standard wiki link, which should stash the potential
        // diary as the path
        let (input, link) = wiki_link(input)?;

        let path = link.path.to_str().ok_or_else(|| {
            nom::Err::Error(VimwikiNomError::from_ctx(&input, "Not diary link"))
        })?;

        // Second, check if the link is a diary
        match parse_date_from_path(path) {
            Some(date) => Ok((
                input,
                link.map(|c| DiaryLink::new(date, c.description, c.anchor)),
            )),
            _ => Err(nom::Err::Error(VimwikiNomError::from_ctx(
                &input,
                "Not diary link",
            ))),
        }
    }

    context("Diary Link", inner)(input)
}

#[inline]
fn parse_date_from_path(path: &str) -> Option<NaiveDate> {
    preceded(tag("diary:"), take_line_while1(anychar))(Span::from(path))
        .ok()
        .map(|x| {
            let date_str = x.1.fragment_str();
            NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()
        })
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::super::elements::{Anchor, Description};
    use super::*;

    #[test]
    fn diary_link_should_fail_if_not_using_diary_scheme() {
        let input = Span::from("[[notdiary:2012-03-05]]");
        assert!(diary_link(input).is_err());
    }

    #[test]
    fn diary_link_should_fail_if_not_using_correct_date_format() {
        let input = Span::from("[[diary:2012/03/05]]");
        assert!(diary_link(input).is_err());
    }

    #[test]
    fn diary_link_should_support_diary_scheme() {
        let input = Span::from("[[diary:2012-03-05]]");
        let (input, link) =
            diary_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.fragment().is_empty());

        assert_eq!(link.date, NaiveDate::from_ymd(2012, 03, 05));
        assert_eq!(link.description, None);
        assert_eq!(link.anchor, None);
    }

    #[test]
    fn diary_link_should_support_a_description() {
        let input = Span::from("[[diary:2012-03-05|some description]]");
        let (input, link) =
            diary_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.fragment().is_empty());

        assert_eq!(link.date, NaiveDate::from_ymd(2012, 03, 05));
        assert_eq!(
            link.description,
            Some(Description::from("some description".to_string()))
        );
        assert_eq!(link.anchor, None);
    }

    #[test]
    fn diary_link_should_support_an_anchor() {
        let input = Span::from("[[diary:2012-03-05#Tomorrow]]");
        let (input, link) =
            diary_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.fragment().is_empty());

        assert_eq!(link.date, NaiveDate::from_ymd(2012, 03, 05));
        assert_eq!(link.description, None,);
        assert_eq!(
            link.anchor,
            Some(Anchor::new(vec!["Tomorrow".to_string()]))
        );
    }

    #[test]
    fn diary_link_should_support_an_anchor_and_description() {
        let input =
            Span::from("[[diary:2012-03-05#Tomorrow|Tasks for tomorrow]]");
        let (input, link) =
            diary_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.fragment().is_empty());

        assert_eq!(link.date, NaiveDate::from_ymd(2012, 03, 05));
        assert_eq!(
            link.description,
            Some(Description::Text("Tasks for tomorrow".to_string()))
        );
        assert_eq!(
            link.anchor,
            Some(Anchor::new(vec!["Tomorrow".to_string()]))
        );
    }
}
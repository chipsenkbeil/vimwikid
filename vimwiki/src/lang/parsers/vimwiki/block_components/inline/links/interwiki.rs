use super::{
    components::{IndexedInterWikiLink, InterWikiLink, NamedInterWikiLink},
    utils::{context, take_line_while1, VimwikiNomError},
    wiki::wiki_link,
    Span, VimwikiIResult, LC,
};
use nom::{bytes::complete::tag, combinator::not, sequence::delimited};
use std::path::PathBuf;

#[inline]
pub fn inter_wiki_link(input: Span) -> VimwikiIResult<LC<InterWikiLink>> {
    fn inner(input: Span) -> VimwikiIResult<LC<InterWikiLink>> {
        let (input, mut link) = wiki_link(input)?;
        let path = link.path.to_str().ok_or_else(|| {
            nom::Err::Error(VimwikiNomError::from_ctx(
                &input,
                "Not interwiki link",
            ))
        })?;

        if let Some((path, index)) = parse_index_from_path(path) {
            // Update path of link after removal of prefix
            link.path = PathBuf::from(path.fragment_str());

            return Ok((
                input,
                link.map(|c| {
                    InterWikiLink::from(IndexedInterWikiLink::new(index, c))
                }),
            ));
        }

        if let Some((path, name)) = parse_name_from_path(path) {
            // Update path of link after removal of prefix
            link.path = PathBuf::from(path.fragment_str());

            return Ok((
                input,
                link.map(|c| {
                    InterWikiLink::from(NamedInterWikiLink::new(name, c))
                }),
            ));
        }

        Err(nom::Err::Error(VimwikiNomError::from_ctx(
            &input,
            "not interwiki link",
        )))
    }

    context("Inter Wiki Link", inner)(input)
}

fn parse_index_from_path(path: &str) -> Option<(Span, u32)> {
    delimited(tag("wiki"), take_line_while1(not(tag(":"))), tag(":"))(
        Span::from(path),
    )
    .ok()
    .map(|(path, index)| {
        index
            .fragment_str()
            .parse::<u32>()
            .ok()
            .map(move |n| (path, n))
    })
    .flatten()
}

fn parse_name_from_path(path: &str) -> Option<(Span, String)> {
    delimited(tag("wn."), take_line_while1(not(tag(":"))), tag(":"))(
        Span::from(path),
    )
    .ok()
    .map(|(path, name)| (path, name.fragment_str().to_string()))
}

#[cfg(test)]
mod tests {
    use super::super::components::{Anchor, Description};
    use super::*;

    #[test]
    fn inter_wiki_link_with_index_should_support_numbered_prefix() {
        let input = Span::from("[[wiki1:This is a link]]");
        let (input, link) = inter_wiki_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.index(), Some(1), "Wrong index detected");
        assert_eq!(link.path().to_path_buf(), PathBuf::from("This is a link"));
        assert_eq!(link.description(), None);
        assert_eq!(link.anchor(), None);
    }

    #[test]
    fn inter_wiki_link_with_index_should_support_description() {
        let input = Span::from(
            "[[wiki1:This is a link source|Description of the link]]",
        );
        let (input, link) = inter_wiki_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.index(), Some(1), "Wrong index detected");
        assert_eq!(
            link.path().to_path_buf(),
            PathBuf::from("This is a link source")
        );
        assert_eq!(
            link.description(),
            Some(&Description::from("Description of the link"))
        );
        assert_eq!(link.anchor(), None);
    }

    #[test]
    fn inter_wiki_link_with_index_should_support_anchors() {
        let input = Span::from("[[wiki1:This is a link source#anchor]]");
        let (input, link) = inter_wiki_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.index(), Some(1), "Wrong index detected");
        assert_eq!(
            link.path().to_path_buf(),
            PathBuf::from("This is a link source")
        );
        assert_eq!(link.description(), None,);
        assert_eq!(link.anchor(), Some(&Anchor::from("anchor")));
    }

    #[test]
    fn inter_wiki_link_with_index_should_support_description_and_anchors() {
        let input = Span::from(
            "[[wiki1:This is a link source#anchor|Description of the link]]",
        );
        let (input, link) = inter_wiki_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.index(), Some(1), "Wrong index detected");
        assert_eq!(
            link.path().to_path_buf(),
            PathBuf::from("This is a link source")
        );
        assert_eq!(
            link.description(),
            Some(&Description::from("Description of the link"))
        );
        assert_eq!(link.anchor(), Some(&Anchor::from("anchor")));
    }

    #[test]
    fn inter_wiki_link_with_name_should_support_named_wikis() {
        let input = Span::from("[[wn.My Name:This is a link]]");
        let (input, link) = inter_wiki_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.name(), Some("My Name"), "Wrong name detected");
        assert_eq!(link.path().to_path_buf(), PathBuf::from("This is a link"));
        assert_eq!(link.description(), None);
        assert_eq!(link.anchor(), None);
    }

    #[test]
    fn inter_wiki_link_with_name_should_support_description() {
        let input =
            Span::from("[[wn.My Name:This is a link|Description of the link]]");
        let (input, link) = inter_wiki_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.name(), Some("My Name"), "Wrong name detected");
        assert_eq!(link.path().to_path_buf(), PathBuf::from("This is a link"));
        assert_eq!(
            link.description(),
            Some(&Description::from("Description of the link"))
        );
        assert_eq!(link.anchor(), None);
    }

    #[test]
    fn inter_wiki_link_with_name_should_support_anchors() {
        let input = Span::from("[[wn.My Name:This is a link#anchor]]");
        let (input, link) = inter_wiki_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.name(), Some("My Name"), "Wrong name detected");
        assert_eq!(link.path().to_path_buf(), PathBuf::from("This is a link"));
        assert_eq!(link.description(), None);
        assert_eq!(link.anchor(), Some(&Anchor::from("anchor")));
    }

    #[test]
    fn inter_wiki_link_with_name_should_support_description_and_anchors() {
        let input = Span::from(
            "[[wn.My Name:This is a link#anchor|Description of the link]]",
        );
        let (input, link) = inter_wiki_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.name(), Some("My Name"), "Wrong name detected");
        assert_eq!(link.path().to_path_buf(), PathBuf::from("This is a link"));
        assert_eq!(
            link.description(),
            Some(&Description::from("Description of the link"))
        );
        assert_eq!(link.anchor(), Some(&Anchor::from("anchor")));
    }
}
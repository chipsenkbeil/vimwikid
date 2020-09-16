use super::{
    components::RawLink,
    utils::{position, uri},
    Span, VimwikiIResult, LC,
};
use nom::combinator::verify;

#[inline]
pub fn raw_link(input: Span) -> VimwikiIResult<LC<RawLink>> {
    let (input, pos) = position(input)?;

    // This will match any URI, but we only want to allow a certain set
    // to ensure that we don't mistake some text preceding a tag
    let (input, uri) = verify(uri, |uri| {
        vec!["http", "https", "ftp", "file", "local", "mailto"]
            .contains(&uri.scheme().as_str())
    })(input)?;

    Ok((input, LC::from((RawLink::from(uri), pos, input))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::utils::new_span;

    #[test]
    fn raw_link_should_support_http_scheme() {
        let input = new_span("http://example.com");
        let (input, link) = raw_link(input).expect("Failed to parse uri");

        // Link should be consumed
        assert!(input.fragment().is_empty());

        assert_eq!(link.uri.scheme(), "http");
        assert_eq!(link.uri.host().unwrap().to_string(), "example.com");
    }

    #[test]
    fn raw_link_should_support_https_scheme() {
        let input = new_span("https://example.com");
        let (input, link) = raw_link(input).expect("Failed to parse uri");

        // Link should be consumed
        assert!(input.fragment().is_empty());

        assert_eq!(link.uri.scheme(), "https");
        assert_eq!(link.uri.host().unwrap().to_string(), "example.com");
    }

    #[test]
    fn raw_link_should_support_no_scheme_with_www() {
        let input = new_span("www.example.com");
        let (input, link) = raw_link(input).expect("Failed to parse uri");

        // Link should be consumed
        assert!(input.fragment().is_empty());

        assert_eq!(link.uri.scheme(), "https");
        assert_eq!(link.uri.host().unwrap().to_string(), "www.example.com");
    }

    #[test]
    fn raw_link_should_support_ftp_scheme() {
        let input = new_span("ftp://example.com");
        let (input, link) = raw_link(input).expect("Failed to parse uri");

        // Link should be consumed
        assert!(input.fragment().is_empty());

        assert_eq!(link.uri.scheme(), "ftp");
        assert_eq!(link.uri.host().unwrap().to_string(), "example.com");
    }

    #[test]
    fn raw_link_should_support_file_scheme() {
        let input = new_span("file:///some/path");
        let (input, link) = raw_link(input).expect("Failed to parse uri");

        // Link should be consumed
        assert!(input.fragment().is_empty());

        assert_eq!(link.uri.scheme(), "file");
        assert_eq!(link.uri.path(), "/some/path");
    }

    #[test]
    fn raw_link_should_support_local_scheme() {
        let input = new_span("local:///some/path");
        let (input, link) = raw_link(input).expect("Failed to parse uri");

        // Link should be consumed
        assert!(input.fragment().is_empty());

        assert_eq!(link.uri.scheme(), "local");
        assert_eq!(link.uri.path(), "/some/path");
    }

    #[test]
    fn raw_link_should_support_mailto_scheme() {
        let input = new_span("mailto:person@example.com");
        let (input, link) = raw_link(input).expect("Failed to parse uri");

        // Link should be consumed
        assert!(input.fragment().is_empty());

        assert_eq!(link.uri.scheme(), "mailto");
        assert_eq!(link.uri.path(), "person@example.com");
    }
}

use super::{Region, Span, VimwikiIResult, VimwikiNomError, LE};
use memchr::{memchr, memchr_iter};
use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_while},
    character::complete::{anychar, crlf, line_ending, space0, space1},
    combinator::{map_res, not, recognize, rest, rest_len, value, verify},
    multi::{many0, many1},
    sequence::{pair, preceded, terminated},
    AsBytes, InputLength, InputTake,
};
use std::convert::TryFrom;
use std::ops::Range;
use uriparse::URI;

/// Wraps a parser in a contextual label, which makes it easier to identify
/// where parsing failures occur
#[cfg(not(feature = "timekeeper"))]
pub use nom::error::context;

/// Wraps a parser in a contextual label, which makes it easier to identify
/// where parsing failures occur. This implementation also logs to a
/// timekeeper table, which can be printed out to evaluate the time spent
/// within each parser wrapped in a context.
#[cfg(feature = "timekeeper")]
pub fn context<'a, T>(
    ctx: &'static str,
    f: impl Fn(Span<'a>) -> VimwikiIResult<T>,
) -> impl Fn(Span<'a>) -> VimwikiIResult<T> {
    crate::timekeeper::parsers::context(ctx, f)
}

/// Parser that wraps another parser's output in a LocatedElement based on
/// the consumed input
#[inline]
pub fn le<'a, T>(
    parser: impl Fn(Span<'a>) -> VimwikiIResult<T>,
) -> impl Fn(Span<'a>) -> VimwikiIResult<LE<T>> {
    use nom::{Offset, Slice};
    context("LE", move |input: Span| {
        let start_line = input.line();
        let start_column = input.column();

        let (input2, x) = parser(input)?;

        // Get offset at end (new start - 1)
        let mut offset = input.offset(&input2);
        if offset > 0 {
            offset -= 1;
        }

        let input = input.slice(offset..);
        let end_line = input.line();
        let end_column = input.column();

        Ok((
            input2,
            LE::new(
                x,
                Region::from((start_line, start_column, end_line, end_column)),
            ),
        ))
    })
}

/// Parser that unwraps another parser's output of LocatedElement into the
/// underlying element
pub fn unwrap_le<'a, T>(
    parser: impl Fn(Span<'a>) -> VimwikiIResult<LE<T>>,
) -> impl Fn(Span<'a>) -> VimwikiIResult<T> {
    context("LE Unwrap", move |input: Span| {
        let (input, le) = parser(input)?;

        Ok((input, le.element))
    })
}

/// Parser that wraps another parser's output in a tuple that also echos out
/// the offset range (starting offset and ending exclusive offset beyond consumed)
#[inline]
pub fn range<'a, T>(
    parser: impl Fn(Span<'a>) -> VimwikiIResult<T>,
) -> impl Fn(Span<'a>) -> VimwikiIResult<(Range<usize>, T)> {
    move |input: Span| {
        let start = input.start_offset();
        let (input, x) = parser(input)?;
        let end = input.start_offset();
        Ok((input, (start..end, x)))
    }
}

/// Parser that will consume an end of line (\n or \r\n) or do nothing if
/// the input is empty
#[inline]
pub fn end_of_line_or_input(input: Span) -> VimwikiIResult<()> {
    fn inner(input: Span) -> VimwikiIResult<()> {
        if input.is_empty() {
            return Ok((input, ()));
        }

        let (input, _) = line_ending(input)?;
        Ok((input, ()))
    }

    context("End of Line/Input", inner)(input)
}

/// Parser that consumes input inside the surrounding left and right sides,
/// failing if not starting with the left or if the right is not found prior
/// to the end of a line. The result is the content WITHIN the surroundings.
/// Will not match right side if it follows immediately from the left.
///
/// Note that the left and right must be non-empty.
pub fn surround_in_line1<'a>(
    left: &'static str,
    right: &'static str,
) -> impl Fn(Span<'a>) -> VimwikiIResult<Span<'a>> {
    fn inner<'a>(
        left: &'static str,
        right: &'static str,
    ) -> impl Fn(Span<'a>) -> VimwikiIResult<Span<'a>> {
        move |input: Span| {
            let (input, _) = tag(left)(input)?;
            let input_bytes = input.as_bytes();

            // First, figure out where our next line will be
            let maybe_newline_pos = memchr(b'\n', input_bytes);

            // Second, look for the starting byte of the right side of our
            // surround wrapper
            for pos in memchr_iter(right.as_bytes()[0], input_bytes) {
                // If we've reached the end of the line, return an error
                if let Some(newline_pos) = maybe_newline_pos {
                    if pos >= newline_pos {
                        return Err(nom::Err::Error(
                            VimwikiNomError::from_ctx(
                                &input,
                                "end of line reached before right side",
                            ),
                        ));
                    }
                }

                // If there would be nothing in the surroundings, continue
                if pos == 0 {
                    continue;
                }

                // Grab everything but the possible start of the right
                let (input, content) = input.take_split(pos);

                // Verify that the right would be next, and if so return our
                // result, otherwise continue
                let (input, right_span) = take(right.len())(input)?;
                if right_span.as_bytes() == right.as_bytes() {
                    return Ok((input, content));
                } else {
                    continue;
                }
            }

            // There was no match of the right side
            Err(nom::Err::Error(VimwikiNomError::from_ctx(
                &input,
                "right side not found",
            )))
        }
    }

    context("Surround in Line", inner(left, right))
}

/// Parser that consumes input while the pattern succeeds or we reach the
/// end of the line. Note that this does NOT consume the line termination.
#[inline]
pub fn take_line_while<'a, T>(
    parser: impl Fn(Span<'a>) -> VimwikiIResult<T>,
) -> impl Fn(Span<'a>) -> VimwikiIResult<Span<'a>> {
    fn single_char<'a, T>(
        parser: impl Fn(Span<'a>) -> VimwikiIResult<T>,
    ) -> impl Fn(Span<'a>) -> VimwikiIResult<char> {
        move |input: Span| {
            let (input, _) = not(end_of_line_or_input)(input)?;

            // NOTE: This is the same as peek(parser), but avoids the issue
            //       of variable being moved out of captured Fn(...)
            let (_, _) = parser(input)?;

            anychar(input)
        }
    }

    context("Take Line While", recognize(many0(single_char(parser))))
}

/// Parser that consumes input while the pattern succeeds or we reach the
/// end of the line. Note that this does NOT consume the line termination.
#[inline]
pub fn take_line_while1<'a, T>(
    parser: impl Fn(Span<'a>) -> VimwikiIResult<T>,
) -> impl Fn(Span<'a>) -> VimwikiIResult<Span<'a>> {
    context(
        "Take Line While 1",
        verify(take_line_while(parser), |s| !s.is_empty()),
    )
}

/// Parser that will consume the remainder of a line (or end of input)
#[inline]
pub fn take_until_end_of_line_or_input(input: Span) -> VimwikiIResult<Span> {
    fn inner(input: Span) -> VimwikiIResult<Span> {
        match memchr(b'\n', input.as_bytes()) {
            Some(pos) => Ok(input.take_split(pos)),
            _ => rest(input),
        }
    }

    context("Take Until End of Line or Input", inner)(input)
}

/// Parser that will consume input until the specified byte is found,
/// consuming the entire input if the byte is not found
#[inline]
pub fn take_until_byte<'a>(
    byte: u8,
) -> impl Fn(Span<'a>) -> VimwikiIResult<Span<'a>> {
    move |input: Span| {
        if let Some(pos) = memchr(byte, input.as_bytes()) {
            Ok(input.take_split(pos))
        } else {
            rest(input)
        }
    }
}

/// Parser that will consume input until the specified byte is found,
/// consuming the entire input if the byte is not found; fails if does
/// not consume at least 1 byte
#[inline]
pub fn take_until_byte1<'a>(
    byte: u8,
) -> impl Fn(Span<'a>) -> VimwikiIResult<Span<'a>> {
    context(
        "Take Until Byte 1",
        verify(take_until_byte(byte), |output| !output.is_empty()),
    )
}

/// Parser that will succeed if input is at the beginning of a line; input
/// will not be consumed
#[inline]
pub fn beginning_of_line(input: Span) -> VimwikiIResult<()> {
    fn inner(input: Span) -> VimwikiIResult<()> {
        let l = input.consumed_len();

        // If we have consumed nothing or the last consumed byte was a newline,
        // we are at the beginning of the line now
        if l == 0 || input.as_consumed()[l - 1] == b'\n' {
            Ok((input, ()))
        } else {
            Err(nom::Err::Error(VimwikiNomError::from_ctx(
                &input,
                "Not at beginning of line",
            )))
        }
    }

    context("Beginning of Line", inner)(input)
}

/// Parser that will consume a line if it is blank, which means that it is
/// comprised of nothing but whitespace and line termination
#[inline]
pub fn blank_line(input: Span) -> VimwikiIResult<String> {
    // 1. We must assert (using span) that we're actually at the beginning of
    //    a line, otherwise this could have been used somewhere after some
    //    other content was matched, and we don't want it to succeed
    //
    // 2. We want to eat up all spaces & tabs on that line, followed by a line
    //    termination. If we happen to be at end of input, then that's okay as
    //    long as there was some space as that would be a blank line at the
    //    end of a file
    context(
        "Blank Line",
        pstring(preceded(
            beginning_of_line,
            alt((
                terminated(space1, end_of_line_or_input),
                terminated(space0, line_ending),
            )),
        )),
    )(input)
}

/// Parser that will consume any line, returning the line's content as output
#[inline]
pub fn any_line(input: Span) -> VimwikiIResult<String> {
    fn inner(input: Span) -> VimwikiIResult<String> {
        let (input, _) = beginning_of_line(input)?;
        let (input, content) = pstring(take_until_end_of_line_or_input)(input)?;
        let (input, _) = end_of_line_or_input(input)?;
        Ok((input, content))
    }

    context("Any Line", inner)(input)
}

/// Parser that consumes a single multispace that could be \r\n, \n, \t, or
/// a space character
#[inline]
pub fn single_multispace(input: Span) -> VimwikiIResult<()> {
    context(
        "Single Multispace",
        value((), alt((crlf, tag("\n"), tag("\t"), tag(" ")))),
    )(input)
}

/// Parser that transforms the output of a parser into an allocated string
#[inline]
pub fn pstring<'a>(
    parser: impl Fn(Span<'a>) -> VimwikiIResult<Span<'a>>,
) -> impl Fn(Span<'a>) -> VimwikiIResult<String> {
    context("Pstring", move |input: Span| {
        let (input, result) = parser(input)?;
        Ok((input, result.as_unsafe_remaining_str().to_string()))
    })
}

/// Parser that scans through the entire input, stepping N across the input
/// using the given step function, applying the provided parser
/// and returning a series of results whenever a parser succeeds; does not
/// consume the input
#[inline]
pub fn scan_with_step<'a, T, U>(
    parser: impl Fn(Span<'a>) -> VimwikiIResult<T>,
    step: impl Fn(Span<'a>) -> VimwikiIResult<U>,
) -> impl Fn(Span<'a>) -> VimwikiIResult<Vec<T>> {
    move |mut input: Span| {
        let mut output = Vec::new();
        let original_input = input;

        loop {
            if let Ok((i, item)) = parser(input) {
                // No advancement happened, so error to prevent infinite loop
                if i == input {
                    return Err(nom::Err::Error(VimwikiNomError::from_ctx(
                        &i,
                        "scan detected infinite loop",
                    )));
                }

                output.push(item);
                input = i;
                continue;
            }

            match step(input) {
                Ok((i, _)) => input = i,
                _ => break,
            }
        }

        Ok((original_input, output))
    }
}

/// Parser that scans through the entire input one character at a time,
/// applying the provided parser and returning a series of results whenever
/// a parser succeeds; does not consume the input
pub fn scan<'a, T>(
    parser: impl Fn(Span<'a>) -> VimwikiIResult<T>,
) -> impl Fn(Span<'a>) -> VimwikiIResult<Vec<T>> {
    scan_with_step(parser, value((), take(1usize)))
}

/// Parser for a general purpose URI.
///
/// ### Regular cases
///
/// 1. https (https://example.com)
/// 2. http (http://example.com)
/// 3. ftp (ftp:)
/// 4. file (file:relative/path)
/// 5. local (local:relative/path)
/// 6. mailto (mailto:someone@example.com)
///
/// ### Special cases
///
/// 1. www (www.example.com) -> (https://www.example.com)
/// 2. // (//some/abs/path) -> (file:/some/abs/path)
#[inline]
pub fn uri(input: Span) -> VimwikiIResult<URI<'static>> {
    // URI = scheme:[//authority]path[?query][#fragment]
    // scheme = sequence of characters beginning with a letter and followed
    //          by any combination of letters, digits, plus (+), period (.),
    //          or hyphen (-)
    // authority = [userinfo@]host[:port] where host is a hostname or IP address
    // path = sequence of path segments separated by / with an empty segment
    //        resulting in //
    let scheme = terminated(
        take_while(|b: u8| {
            let c = char::from(b);
            c.is_alphanumeric() || c == '+' || c == '.' || c == '-'
        }),
        tag(":"),
    );

    // TODO: Do we need to support whitespace in our raw URIs?
    context(
        "URI",
        map_res(
            recognize(pair(
                alt((tag("www."), tag("//"), scheme)),
                many1(pair(not(single_multispace), anychar)),
            )),
            |s| {
                URI::try_from(
                    match s.as_unsafe_remaining_str() {
                        text if text.starts_with("www.") => {
                            ["https://", text].join("")
                        }
                        text if text.starts_with("//") => {
                            ["file:/", text].join("")
                        }
                        text => text.to_string(),
                    }
                    .as_str(),
                )
                .map(|uri| uri.into_owned())
            },
        ),
    )(input)
}

/// Counts the spaces & tabs that are trailing in our input
pub fn count_trailing_whitespace(input: Span) -> VimwikiIResult<usize> {
    fn inner(input: Span) -> VimwikiIResult<usize> {
        let mut cnt = 0;

        // Count whitespace in reverse so we know how many are trailing
        for b in input.as_bytes().iter().rev() {
            if !nom::character::is_space(*b) {
                break;
            }
            cnt += 1;
        }

        Ok((input, cnt))
    }

    context("Count Trailing Whitespace", inner)(input)
}

/// Trims the trailing whitespace from input, essentially working backwards
/// to cut off part of the input
pub fn trim_trailing_whitespace(input: Span) -> VimwikiIResult<()> {
    fn inner(input: Span) -> VimwikiIResult<()> {
        use nom::Slice;
        let (input, len) = rest_len(input)?;
        let (input, cnt) = count_trailing_whitespace(input)?;
        Ok((input.slice(..(len - cnt)), ()))
    }

    context("Trim Trailing Whitespace", inner)(input)
}

/// Trims the leading and trailing whitespace from input
pub fn trim_whitespace(input: Span) -> VimwikiIResult<()> {
    fn inner(input: Span) -> VimwikiIResult<()> {
        let (input, _) = space0(input)?;
        let (input, _) = trim_trailing_whitespace(input)?;
        Ok((input, ()))
    }

    context("Trim Whitespace", inner)(input)
}

/// Takes from the end instead of the beginning
pub fn take_end<'a, C>(
    count: C,
) -> impl Fn(Span<'a>) -> VimwikiIResult<Span<'a>>
where
    C: nom::ToUsize,
{
    use nom::{
        error::{ErrorKind, ParseError},
        Err,
    };
    let cnt = count.to_usize();
    context("Take End", move |input: Span| {
        let len = input.input_len();
        if cnt > len {
            Err(Err::Error(VimwikiNomError::from_error_kind(
                input,
                ErrorKind::Eof,
            )))
        } else {
            let (end, input) = input.take_split(len - cnt);
            Ok((input, end))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::character::complete::char;

    #[inline]
    fn take_and_toss(n: usize) -> impl Fn(Span) -> VimwikiIResult<()> {
        move |input: Span| {
            nom::combinator::value((), nom::bytes::complete::take(n))(input)
        }
    }

    #[test]
    fn end_of_line_or_input_should_succeed_if_line_ending() {
        assert!(end_of_line_or_input(Span::from("\n")).is_ok());
        assert!(end_of_line_or_input(Span::from("\r\n")).is_ok());
    }

    #[test]
    fn end_of_line_or_input_should_succeed_if_input_empty() {
        assert!(end_of_line_or_input(Span::from("")).is_ok());
    }

    #[test]
    fn beginning_of_line_should_fail_if_not_at_beginning_of_line() {
        let input = Span::from("1234");
        let (input, _) =
            take_and_toss(1)(input).expect("Failed to take a character");
        assert!(beginning_of_line(input).is_err());
    }

    #[test]
    fn beginning_of_line_should_succeed_if_at_beginning_of_first_line() {
        let input = Span::from("1234");
        let (input, _) = beginning_of_line(input)
            .expect("Unexpectedly think not at beginning of line");

        // Input shouldn't be consumed
        assert_eq!(input.as_unsafe_remaining_str(), "1234");
    }

    #[test]
    fn beginning_of_line_should_succeed_if_at_beginning_of_any_line() {
        let input = Span::from("abc\n1234");
        let (input, _) =
            take_and_toss(4)(input).expect("Failed to take a character");
        let (input, _) = beginning_of_line(input)
            .expect("Unexpectedly think not at beginning of line");

        // Input shouldn't be consumed
        assert_eq!(input.as_unsafe_remaining_str(), "1234");
    }

    #[test]
    fn blank_line_should_fail_if_line_contains_non_whitespace() {
        let input = Span::from("1234");
        assert!(blank_line(input).is_err());
    }

    #[test]
    fn blank_line_should_fail_if_input_empty_and_at_beginning_of_line() {
        let input = Span::from("");
        assert!(blank_line(input).is_err());
    }
    #[test]
    fn blank_line_should_succeed_if_has_whitespace_but_no_line_termination() {
        let input = Span::from(" ");
        let (input, s) = blank_line(input).expect("Failed to parse blank line");
        assert!(input.is_empty(), "Did not consume blank line");
        assert_eq!(s, " ");
    }

    #[test]
    fn blank_line_should_succeed_if_line_empty() {
        let input = Span::from("\nabcd");
        let (input, _) = blank_line(input).expect("Failed to parse blank line");

        // Line including termination should be consumed
        assert_eq!(input.as_unsafe_remaining_str(), "abcd");
    }

    #[test]
    fn blank_line_should_succeed_if_line_only_has_whitespace() {
        let input = Span::from(" \t\nabcd");
        let (input, _) = blank_line(input).expect("Failed to parse blank line");

        // Line including termination should be consumed
        assert_eq!(input.as_unsafe_remaining_str(), "abcd");
    }

    #[test]
    fn blank_line_should_succeed_if_on_last_line_and_only_whitespace() {
        let input = Span::from(" \t");
        let (input, _) = blank_line(input).expect("Failed to parse blank line");

        // Line including termination should be consumed
        assert_eq!(input.as_unsafe_remaining_str(), "");
    }

    #[test]
    fn any_line_should_fail_if_not_at_beginning_of_line() {
        let input = Span::from("abc");
        let (input, _) =
            take_and_toss(1)(input).expect("Failed to take a character");
        assert!(any_line(input).is_err());
    }

    #[test]
    fn any_line_should_return_empty_if_nothing_in_line() {
        let input = Span::from("\nabcd");
        let (input, content) =
            any_line(input).expect("Failed to parse any line");
        assert_eq!(input.as_unsafe_remaining_str(), "abcd");
        assert!(content.is_empty());
    }

    #[test]
    fn any_line_should_return_all_content_update_to_newline() {
        let input = Span::from("test\nabcd");
        let (input, line) = any_line(input).expect("Failed to parse any line");
        assert_eq!(input.as_unsafe_remaining_str(), "abcd");
        assert_eq!(line, "test");
    }

    #[test]
    fn any_line_should_return_all_content_remaining_if_no_more_newline() {
        let input = Span::from("test");
        let (input, line) = any_line(input).expect("Failed to parse any line");
        assert_eq!(input.as_unsafe_remaining_str(), "");
        assert_eq!(line, "test");
    }

    #[test]
    fn single_multispace_should_fail_if_input_empty() {
        let input = Span::from("");
        assert!(single_multispace(input).is_err());
    }

    #[test]
    fn single_multispace_should_fail_if_not_multispace_character() {
        let input = Span::from("a");
        assert!(single_multispace(input).is_err());
    }

    #[test]
    fn single_multispace_should_succeed_if_tab() {
        let input = Span::from("\t abc");
        let (input, _) = single_multispace(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), " abc");
    }

    #[test]
    fn single_multispace_should_succeed_if_space() {
        let input = Span::from("  abc");
        let (input, _) = single_multispace(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), " abc");
    }

    #[test]
    fn single_multispace_should_succeed_if_crlf() {
        let input = Span::from("\r\n abc");
        let (input, _) = single_multispace(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), " abc");
    }

    #[test]
    fn single_multispace_should_succeed_if_newline() {
        let input = Span::from("\n abc");
        let (input, _) = single_multispace(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), " abc");
    }

    #[test]
    fn uri_should_fail_if_input_empty() {
        let input = Span::from("");
        assert!(uri(input).is_err());
    }

    #[test]
    fn uri_should_fail_if_no_scheme_and_not_www_or_absolute_path() {
        let input = Span::from("example.com");
        assert!(uri(input).is_err());
    }

    #[test]
    fn uri_should_succeed_if_starts_with_www_and_will_add_https_as_scheme() {
        let input = Span::from("www.example.com");
        let (input, u) = uri(input).expect("Failed to parse uri");
        assert!(input.is_empty());
        assert_eq!(u.scheme(), "https");
        assert_eq!(u.host().unwrap().to_string(), "www.example.com");
    }

    #[test]
    fn uri_should_succeed_if_starts_with_absolute_path_and_will_add_file_as_scheme(
    ) {
        let input = Span::from("//some/absolute/path");
        let (input, u) = uri(input).expect("Failed to parse uri");
        assert!(input.is_empty());
        assert_eq!(u.scheme(), "file");
        assert_eq!(u.path(), "/some/absolute/path");
    }

    #[test]
    fn uri_should_succeed_if_starts_with_scheme() {
        let input = Span::from("https://github.com/vimwiki/vimwiki.git");
        let (input, u) = uri(input).expect("Failed to parse uri");
        assert!(input.is_empty());
        assert_eq!(u.scheme(), "https");
        assert_eq!(u.host().unwrap().to_string(), "github.com");
        assert_eq!(u.path(), "/vimwiki/vimwiki.git");

        let input = Span::from("mailto:habamax@gmail.com");
        let (input, u) = uri(input).expect("Failed to parse uri");
        assert!(input.is_empty());
        assert_eq!(u.scheme(), "mailto");
        assert_eq!(u.path(), "habamax@gmail.com");

        let input = Span::from("ftp://vim.org");
        let (input, u) = uri(input).expect("Failed to parse uri");
        assert!(input.is_empty());
        assert_eq!(u.scheme(), "ftp");
        assert_eq!(u.host().unwrap().to_string(), "vim.org");
    }

    #[test]
    fn take_line_while_should_yield_empty_if_empty_input() {
        let input = Span::from("");
        let (_, taken) = take_line_while(anychar)(input).unwrap();
        assert_eq!(taken.as_unsafe_remaining_str(), "");
    }

    #[test]
    fn take_line_while_should_yield_empty_if_line_termination_next() {
        let input = Span::from("\nabcd");
        let (input, taken) = take_line_while(anychar)(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "\nabcd");
        assert_eq!(taken.as_unsafe_remaining_str(), "");

        let input = Span::from("\r\nabcd");
        let (input, taken) = take_line_while(anychar)(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "\r\nabcd");
        assert_eq!(taken.as_unsafe_remaining_str(), "");
    }

    #[test]
    fn take_line_while_should_yield_empty_if_stops_without_ever_succeeding() {
        let input = Span::from("aabb\nabcd");
        let (input, taken) = take_line_while(char('c'))(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "aabb\nabcd");
        assert_eq!(taken.as_unsafe_remaining_str(), "");
    }

    #[test]
    fn take_line_while_should_take_until_provided_parser_fails() {
        let input = Span::from("aabb\nabcd");
        let (input, taken) = take_line_while(char('a'))(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "bb\nabcd");
        assert_eq!(taken.as_unsafe_remaining_str(), "aa");
    }

    #[test]
    fn take_line_while_should_take_until_line_termination_reached() {
        let input = Span::from("aabb\nabcd");
        let (input, taken) = take_line_while(anychar)(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "\nabcd");
        assert_eq!(taken.as_unsafe_remaining_str(), "aabb");
    }

    #[test]
    fn take_line_while_should_count_condition_parser_towards_consumption() {
        // NOTE: Using an ODD number of characters as otherwise we wouldn't
        //       catch the error which was happening where we would use the
        //       parser, char('-'), which would consume a character since it
        //       was not a not(...) and then try to use an anychar, so we
        //       would end up consuming TWO parsers instead of one
        let input = Span::from("-----");
        let (input, taken) = take_line_while(char('-'))(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "");
        assert_eq!(taken.as_unsafe_remaining_str(), "-----");
    }

    #[test]
    fn take_line_while1_should_fail_if_empty_input() {
        let input = Span::from("");
        assert!(take_line_while1(anychar)(input).is_err());
    }

    #[test]
    fn take_line_while1_should_fail_if_line_termination_next() {
        let input = Span::from("\nabcd");
        assert!(take_line_while1(anychar)(input).is_err());

        let input = Span::from("\r\nabcd");
        assert!(take_line_while1(anychar)(input).is_err());
    }

    #[test]
    fn take_line_while1_should_fail_if_stops_without_ever_succeeding() {
        let input = Span::from("aabb\nabcd");
        assert!(take_line_while1(char('c'))(input).is_err());
    }

    #[test]
    fn take_line_while1_should_take_until_provided_parser_fails() {
        let input = Span::from("aabb\nabcd");
        let (input, taken) = take_line_while1(char('a'))(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "bb\nabcd");
        assert_eq!(taken.as_unsafe_remaining_str(), "aa");
    }

    #[test]
    fn take_line_while1_should_take_until_line_termination_reached() {
        let input = Span::from("aabb\nabcd");
        let (input, taken) = take_line_while1(anychar)(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "\nabcd");
        assert_eq!(taken.as_unsafe_remaining_str(), "aabb");
    }

    #[test]
    fn take_line_while1_should_count_condition_parser_towards_consumption() {
        // NOTE: Using an ODD number of characters as otherwise we wouldn't
        //       catch the error which was happening where we would use the
        //       parser, char('-'), which would consume a character since it
        //       was not a not(...) and then try to use an anychar, so we
        //       would end up consuming TWO parsers instead of one
        let input = Span::from("-----");
        let (input, taken) = take_line_while1(char('-'))(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "");
        assert_eq!(taken.as_unsafe_remaining_str(), "-----");
    }

    #[test]
    fn scan_should_fail_if_no_advancement_is_made_with_parser() {
        let input = Span::from("aaa");
        assert!(scan(not(char('b')))(input).is_err());
    }

    #[test]
    fn scan_should_yield_an_empty_vec_if_input_empty() {
        let input = Span::from("");
        let (_, results) = scan(char('a'))(input).unwrap();
        assert!(results.is_empty(), "Unexpectedly found parser results");
    }

    #[test]
    fn scan_should_consume_all_input() {
        let input = Span::from("abc");
        let (input, _) = scan(char('a'))(input).unwrap();
        assert!(input.is_empty(), "scan did not consume all input");
    }

    #[test]
    fn scan_should_yield_an_empty_vec_if_parser_never_succeeds() {
        let input = Span::from("bbb");
        let (input, results) = scan(char('a'))(input).unwrap();
        assert!(input.is_empty(), "scan did not consume all input");
        assert!(results.is_empty(), "Unexpectedly found results");
    }

    #[test]
    fn scan_should_yield_a_vec_containing_all_of_parser_successes() {
        let input = Span::from("aba");
        let (input, results) = scan(char('a'))(input).unwrap();
        assert!(input.is_empty(), "scan did not consume all input");
        assert_eq!(results, vec!['a', 'a']);
    }

    #[test]
    fn range_should_include_the_starting_and_ending_offset_of_consumed_parser()
    {
        let input = Span::from("aba");
        let (input, (r, results)) = range(take(2usize))(input).unwrap();
        assert_eq!(
            input.as_unsafe_remaining_str(),
            "a",
            "offset did not consume expected input"
        );
        assert_eq!(r.start, 0, "Start was wrong position");
        assert_eq!(r.end, 2, "End was wrong position");
        assert_eq!(
            results.as_unsafe_remaining_str(),
            "ab",
            "Parser did not function properly"
        );
    }
}

//! Superset composition: a miniature JSON5-style grammar built from this crate's public token
//! rules, proving that the lifting chain continues one level up — JSON rules lifted into
//! [`JsoncToken`] lift again into a wider kind, alongside the comment rules.
//!
//! The pattern mirrors how this crate composes on `strata-json-rules`: define a wider token kind
//! wrapping [`JsoncToken`], lift each kept rule with `map_kind`, and add the rules that differ
//! (here, a single-quoted string).

use std::io::Cursor;
use strata_json_rules::{
    JsonToken, LiteralRule, NumberRule, PunctRule, StringRule, WhitespaceRule,
};
use strata_jsonc_rules::{BlockCommentRule, JsoncToken, LineCommentRule};
use strata_lexer::{
    LexError, LexErrorKind, Lexer, Lookahead, Match, Matcher, MatcherExt, MatcherVec, SpecScanner,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SupersetToken {
    Jsonc(JsoncToken),
    SingleQuotedString,
    Eof,
}

/// A `'...'` single-quoted string rule (no escapes) — the one shape this superset adds over
/// JSONC.
#[derive(Debug, Clone, Copy, Default)]
struct SingleQuoteStringRule;

impl Matcher for SingleQuoteStringRule {
    type Kind = SupersetToken;

    fn try_match(
        &self,
        look: &mut Lookahead<'_>,
    ) -> Result<Option<Match<SupersetToken>>, LexError> {
        if look.peek_byte()? != Some(b'\'') {
            return Ok(None);
        }

        let start = look.position();
        let mut index: usize = 1;

        loop {
            let bytes = look.peek(index.saturating_add(1))?;

            match bytes.get(index) {
                None => return Err(LexError::unterminated_delimiter(start)),
                Some(b'\'') => {
                    return Ok(Match::from_len(
                        SupersetToken::SingleQuotedString,
                        index.saturating_add(1),
                    ));
                }
                Some(_) => index = index.saturating_add(1),
            }
        }
    }
}

fn superset_rules() -> MatcherVec<SupersetToken> {
    let json: fn(JsonToken) -> JsoncToken = JsoncToken::Json;
    let jsonc: fn(JsoncToken) -> SupersetToken = SupersetToken::Jsonc;

    vec![
        SingleQuoteStringRule.boxed(),
        // The comment rules this crate adds lift once...
        LineCommentRule.map_kind(jsonc).boxed(),
        BlockCommentRule.map_kind(jsonc).boxed(),
        // ...and the JSON rules lift twice: JsonToken -> JsoncToken -> SupersetToken.
        WhitespaceRule.map_kind(json).map_kind(jsonc).boxed(),
        PunctRule.map_kind(json).map_kind(jsonc).boxed(),
        LiteralRule.map_kind(json).map_kind(jsonc).boxed(),
        StringRule.map_kind(json).map_kind(jsonc).boxed(),
        NumberRule.map_kind(json).map_kind(jsonc).boxed(),
        // BomRule deliberately omitted: this superset simply does not know the shape.
    ]
}

fn lex_kinds(input: &str) -> Result<Vec<SupersetToken>, LexError> {
    let scanner = SpecScanner::new(superset_rules(), SupersetToken::Eof);
    let mut lexer = Lexer::new(Cursor::new(input.as_bytes().to_vec()), scanner);
    let mut kinds = Vec::new();

    loop {
        let token = lexer.next_token()?;
        let done = token.kind == SupersetToken::Eof;
        kinds.push(token.kind);

        if done {
            return Ok(kinds);
        }
    }
}

#[test]
fn reused_jsonc_rules_lex_under_the_wider_kind() -> Result<(), LexError> {
    let kinds = lex_kinds("{\"a\": 1} // note")?;

    assert_eq!(
        kinds,
        vec![
            SupersetToken::Jsonc(JsoncToken::Json(JsonToken::LeftBrace)),
            SupersetToken::Jsonc(JsoncToken::Json(JsonToken::String)),
            SupersetToken::Jsonc(JsoncToken::Json(JsonToken::Colon)),
            SupersetToken::Jsonc(JsoncToken::Json(JsonToken::Whitespace)),
            SupersetToken::Jsonc(JsoncToken::Json(JsonToken::Number)),
            SupersetToken::Jsonc(JsoncToken::Json(JsonToken::RightBrace)),
            SupersetToken::Jsonc(JsoncToken::Json(JsonToken::Whitespace)),
            SupersetToken::Jsonc(JsoncToken::LineComment),
            SupersetToken::Eof,
        ]
    );

    Ok(())
}

#[test]
fn added_rule_extends_the_grammar() -> Result<(), LexError> {
    let kinds = lex_kinds("'hi' /* note */ null")?;

    assert_eq!(
        kinds,
        vec![
            SupersetToken::SingleQuotedString,
            SupersetToken::Jsonc(JsoncToken::Json(JsonToken::Whitespace)),
            SupersetToken::Jsonc(JsoncToken::BlockComment),
            SupersetToken::Jsonc(JsoncToken::Json(JsonToken::Whitespace)),
            SupersetToken::Jsonc(JsoncToken::Json(JsonToken::Null)),
            SupersetToken::Eof,
        ]
    );

    Ok(())
}

#[test]
fn comment_validation_survives_the_lift() {
    // The lifted block-comment rule still reports an unclosed comment at its opener.
    let Err(error) = lex_kinds("null /* open") else {
        panic!("expected the lifted block-comment rule to keep rejecting unclosed comments");
    };
    assert!(matches!(error.kind, LexErrorKind::UnterminatedDelimiter));
    assert_eq!(error.position.byte(), 5);
}

#[test]
fn json_shapes_stay_strict_inside_the_superset() {
    // Two lifts deep, the reused number rule keeps its validation: `01` is still a lexical
    // error.
    let Err(error) = lex_kinds("01") else {
        panic!("expected the reused number rule to keep rejecting leading zeros");
    };
    assert!(matches!(error.kind, LexErrorKind::Unrecognized));
    assert_eq!(error.position.byte(), 1);
}

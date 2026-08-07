//! The JSONC token kind enum.

use strata_json_rules::JsonToken;

/// Token kinds produced by the JSONC token rules.
///
/// Every significant JSON shape is carried unchanged inside [`JsoncToken::Json`]; the two comment
/// kinds are the only additions, and both are trivia — JSONC adds no significant token kinds
/// (jsonc.org: "Removing all comments MUST yield the same data representation of the JSON
/// values"). A further superset wraps this enum in its own kind the same way this crate wraps
/// [`JsonToken`], lifting the rules with
/// [`MatcherExt::map_kind`](strata_lexer::MatcherExt::map_kind).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JsoncToken {
    /// A token shared with RFC 8259 JSON, produced by a lifted `strata-json-rules` rule.
    Json(JsonToken),
    /// A `//` line comment, ending before the line break (or at end of input).
    LineComment,
    /// A `/* */` block comment, span covering both delimiters. Block comments do not nest.
    BlockComment,
    /// End of input.
    Eof,
}

impl JsoncToken {
    /// Returns whether this kind is trivia — grammatically insignificant source content that the
    /// CST retains instead of discarding. Both comment kinds are trivia, alongside JSON's
    /// whitespace and byte-order mark.
    pub const fn is_trivia(self) -> bool {
        match self {
            Self::Json(token) => token.is_trivia(),
            Self::LineComment | Self::BlockComment => true,
            Self::Eof => false,
        }
    }

    /// A short human-readable description for diagnostics.
    pub const fn describe(self) -> &'static str {
        match self {
            Self::Json(token) => token.describe(),
            Self::LineComment => "a line comment",
            Self::BlockComment => "a block comment",
            Self::Eof => "end of input",
        }
    }
}

//! JSONC (jsonc.org "JSON with Comments") token rules for the Strata parser ecosystem.
//!
//! JSONC extends RFC 8259 only at the lexical level: `//` line comments and non-nesting `/* */`
//! block comments may appear anywhere insignificant whitespace is allowed. This crate therefore
//! reuses `strata-json-rules`' rule structs unchanged — lifted into [`JsoncToken`] via
//! [`MatcherExt::map_kind`](strata_lexer::MatcherExt::map_kind) — and adds [`LineCommentRule`]
//! and [`BlockCommentRule`]. Reused rules keep their validation: `01` is still a lexical error,
//! strings still reject bad escapes, and so on.
//!
//! All rules are exported individually so a further superset can compose on JSONC the same way
//! this crate composes on JSON: wrap [`JsoncToken`] in a wider kind enum, lift the rules it
//! keeps with `map_kind`, and add or replace only what differs.
//!
//! The crate is standalone on purpose: a parser for JSONC (or a superset of it) depends on these
//! rules and its own additions without pulling in a JSON parser.

mod kind;
mod rules;

pub use kind::JsoncToken;
pub use rules::{
    BlockCommentRule, LineCommentRule, StrictRuleSet, strict_rule_set, strict_scanner,
};

// Re-exported so downstream code can name the kinds carried inside `JsoncToken` and share rule
// sets without depending on `strata-json-rules` directly. `SharedRuleSet` is kind-generic
// upstream, so it wraps JSONC rule sets too.
pub use strata_json_rules::{JsonToken, SharedRuleSet};

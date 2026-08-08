# strata-jsonc-rules

`strata-jsonc-rules` is the JSONC ([jsonc.org](https://jsonc.org) "JSON with Comments")
token-rules crate for the Strata parser ecosystem: `strata-json-rules`' RFC 8259 rules reused
unchanged — lifted into a wider kind enum — plus the two comment shapes JSONC adds. It is
standalone on purpose — a parser for JSONC (or a superset of it) depends on just these rules and
adds its own, without pulling in a JSON parser.

The crate sits at the rules layer of the per-language stack:

```text
strata-json-rules -> strata-jsonc-rules (this crate) -> strata-jsonc-parser -> strata-jsonc
```

- **Edition / MSRV:** Rust 2024, MSRV `1.87.0`.
- **License:** MPL-2.0.
- **Dependencies:** `strata-json-rules` and `strata-lexer` (which bring `strata-reader` and
  `strata-lease`).

## 1. Module and public-API map

| Module  | Public items |
|---------|--------------|
| `kind`  | `JsoncToken` (with `is_trivia()`, `describe()`) |
| `rules` | `LineCommentRule`, `BlockCommentRule`, `StrictRuleSet`, `strict_rule_set()`, `strict_scanner()` |

Everything is re-exported at the crate root, alongside `strata_json_rules::JsonToken` (so
downstream code can name the kinds carried inside `JsoncToken` without depending on
`strata-json-rules` directly) and `SharedRuleSet` (kind-generic upstream, so it wraps JSONC rule
sets too).

## 2. Token kinds

`JsoncToken` wraps `strata-json-rules`' kind instead of duplicating it: `Json(JsonToken)` carries
every shared shape unchanged, and `LineComment`, `BlockComment`, and `Eof` are the only
additions. Both comment kinds are trivia (`is_trivia()`) — JSONC adds no *significant* token
kinds, which is jsonc.org's "Removing all comments MUST yield the same data representation of
the JSON values" made visible in the enum. A further superset wraps `JsoncToken` in its own kind
the same way this crate wraps `JsonToken`.

## 3. The rules

The reused JSON rules are lifted with `MatcherExt::map_kind`: `StrictRuleSet` holds the six
strict RFC 8259 rules as `MapMatcher<Rule, fn(JsonToken) -> JsoncToken>` plus the two comment
rules, and implements `MatcherSet` by chaining both groups — fully monomorphic, `Copy`, no
boxing. Reused rules keep their validation: `01` is still a lexical error inside JSONC, strings
still reject bad escapes. `BomRule` stays in the set for the same reason as upstream — the rules
describe which shapes *exist*; the parser owns the BOM policy (reject strictly, or retain as
trivia under `allow_bom`).

The added rules (jsonc.org §"Comments"):

- `LineCommentRule` — `//` up to (excluding) the next CR/LF, or to end of input. "Single-line
  comments start with `//` and continue until a line ending is encountered." The line ending
  itself lexes as ordinary whitespace, keeping blank-line counting uniform; a comment terminated
  by end of input is valid (the ABNF makes the line ending optional).
- `BlockCommentRule` — `/*` to the *first* `*/` ("Multi-line comments cannot be nested. The
  closing of the nested comment will be interpreted as the end of the outer comment."). The span
  covers both delimiters; `/*/` cannot close with the opener's own `*`. An unclosed comment is
  `LexErrorKind::UnterminatedDelimiter` at the opening `/*`.

Both rules validate that comment content is UTF-8 — the jsonc.org ABNF defines content in code
points and RFC 8259 §8.1 requires UTF-8, the same policy `strata-json-rules` applies inside
strings — reporting the offending byte with a line/column-correct position even inside multiline
comments. Raw control characters (other than line endings) are valid comment content per the
ABNF. `#` is matched by no rule ("Single-line comments that start with `#` are not supported";
JSONC must remain a subset of JavaScript syntax) and surfaces as an unrecognized byte.

Trailing commas need no extra rule — `,` is already a JSON token; a parser's
`allow_trailing_commas` option is purely grammatical.

`strict_rule_set()` constructs the strict set and `strict_scanner()` wraps it in a `SpecScanner`
with `JsoncToken::Eof`.

## 4. Composition points for further supersets

The lifting chain continues one level up: a superset of JSONC defines its own kind enum wrapping
`JsoncToken`, lifts each kept rule, and adds or replaces only what differs. The comment rules
lift once; the JSON rules lift twice:

```rust
enum SupersetToken { Jsonc(JsoncToken), SingleQuotedString, Eof }

let json: fn(JsonToken) -> JsoncToken = JsoncToken::Json;
let jsonc: fn(JsoncToken) -> SupersetToken = SupersetToken::Jsonc;

let rules: MatcherVec<SupersetToken> = vec![
    SingleQuoteStringRule.boxed(),                        // the shape the superset adds
    LineCommentRule.map_kind(jsonc).boxed(),              // JSONC's additions, lifted once
    BlockCommentRule.map_kind(jsonc).boxed(),
    WhitespaceRule.map_kind(json).map_kind(jsonc).boxed(), // JSON's rules, lifted twice
    PunctRule.map_kind(json).map_kind(jsonc).boxed(),
    // ...
];
```

Rules to change are simply not included; reused rules keep their validation at any lifting depth
— `01` and unclosed block comments stay lexical errors two lifts deep. `tests/composition.rs` is
the worked example.

## 5. Design choices

- **Reuse by lifting, not forking.** Every shared shape is the upstream rule struct behind a
  kind-mapping wrapper, so JSON-level fixes propagate and the two rule sets cannot drift. What
  is JSONC-specific in this crate is exactly the two comment rules.
- **Validation at the lexical boundary.** Comment termination and UTF-8 are rule concerns with
  exact positions, like upstream's string/number shape checks. Because comments can span lines,
  sub-token error positions are computed by walking the comment bytes with the lexer's own
  line-break rules (CR, LF, and CRLF each count once) instead of advancing on the start line.
- **Line endings stay whitespace.** A line comment's span excludes its terminating CR/LF, so
  line breaks and blank lines are counted in one place (whitespace trivia) whether or not a
  comment precedes them.
- **Rules standalone, policy downstream.** As upstream: the set describes shapes; BOM handling
  and trailing-comma tolerance are parser options.

## 6. Quality bar

Same as the ecosystem: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D
warnings` under the strict lint set in `Cargo.toml`, and `cargo test` all clean. Lint exceptions
use `#[expect(lint, reason = "…")]` on the smallest existing scope.

## 7. Limitations and follow-up

- The public API is not settled; this is an initial-development crate on an experimental branch.
- Package verification (`cargo package`) is blocked until `strata-json-rules` and `strata-lexer`
  are published and the path dependencies gain versions.

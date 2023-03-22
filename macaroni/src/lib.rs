//! Markdown parser for language servers.
//!
//! # Examples
//!
//! ```rust
//! #![feature(assert_matches)]
//!
//! use std::assert_matches::assert_matches;
//!
//! use macaroni::*;
//!
//! let input = "Hello, [world](https://en.wikipedia.org/wiki/World)!";
//!
//! let Document { block_elements, inline_elements } = parse_document(input);
//!
//! assert_eq!(block_elements.len(), 2);
//! assert_matches!(&block_elements[0], BlockElement::Root);
//! assert_matches!(&block_elements[1], BlockElement::Paragraph { .. });
//!
//! if let BlockElement::Paragraph { lines } = &block_elements[1] {
//!   assert_eq!(lines.len(), 1);
//!   assert_eq!(lines[0].start.offset, 0);
//!   assert_eq!(lines[0].end.offset, input.len());
//! }
//!
//! // assert_eq!(inline_elements.len(), 1);
//! // assert_matches!(&inline_elements[0], InlineElement::InlineLink { .. });
//! // let content_range = &inline_elements[0].range;
//! // assert_eq!(
//! //   &input[content_range.start.offset..content_range.end.offset],
//! //   "[world](https://en.wikipedia.org/wiki/World)"
//! // );
//! ```

#![feature(let_chains)]
#![warn(
  clippy::branches_sharing_code,
  clippy::cognitive_complexity,
  clippy::derive_partial_eq_without_eq,
  clippy::empty_line_after_outer_attr,
  clippy::equatable_if_let,
  clippy::fallible_impl_from,
  clippy::manual_clamp,
  clippy::missing_const_for_fn,
  clippy::missing_errors_doc,
  clippy::missing_panics_doc,
  clippy::needless_collect,
  clippy::needless_continue,
  clippy::needless_for_each,
  clippy::needless_pass_by_value,
  clippy::nonstandard_macro_braces,
  clippy::option_if_let_else,
  clippy::or_fun_call,
  clippy::ptr_as_ptr,
  clippy::range_minus_one,
  clippy::range_plus_one,
  clippy::redundant_closure_for_method_calls,
  clippy::redundant_else,
  clippy::redundant_pub_crate,
  clippy::similar_names,
  clippy::single_match_else,
  clippy::string_lit_as_bytes,
  clippy::too_many_lines,
  clippy::trait_duplication_in_bounds,
  clippy::trivial_regex,
  clippy::type_repetition_in_bounds,
  clippy::uninlined_format_args,
  clippy::unnecessary_join,
  clippy::unnecessary_wraps,
  clippy::unnested_or_patterns,
  clippy::unreadable_literal,
  clippy::unused_peekable,
  clippy::unused_self,
  clippy::use_self,
  clippy::used_underscore_binding,
  clippy::useless_let_if_seq
)]
#![deny(clippy::semicolon_if_nothing_returned)]

#[macro_use]
mod macros;
pub mod parser;
pub mod types;
mod utf8;

pub use parser::{parse_block_elements, parse_document, parse_inline_elements, BlockParser, InlineParser};
pub use types::{BlockElement, Document, HeadingLevel, InlineElement, Position, Range};

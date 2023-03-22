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

use std::net::SocketAddr;

use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use tower_http::services::ServeDir;

use macaroni::*;

#[tokio::main]
async fn main() {
  // TODO: Command-line options (e.g. port).

  let public_path = std::path::Path::new(file!()).parent().unwrap().parent().unwrap().join("public");
  let public_path = public_path.to_str().unwrap();

  let app = Router::new().route("/parse", post(parse)).nest_service("/", ServeDir::new(public_path));
  let addr = SocketAddr::from(([127, 0, 0, 1], 4242));

  println!("Listening on http://localhost:4242");
  axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}

async fn parse(Json(payload): Json<ParseRequest>) -> Json<Document> {
  Json(parse_document(&payload.source))
}

#[derive(Deserialize)]
struct ParseRequest {
  source: String,
}

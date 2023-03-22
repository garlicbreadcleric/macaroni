//! Markdown parser.
//!
//! Macaroni mostly follows the parsing strategy outlined in the CommonMark specification
//! (<https://spec.commonmark.org/0.30/#appendix-a-parsing-strategy>). Parsing is implemented in two phases:
//! [block structure](parse_block_elements) and [inline structure](parse_inline_elements).

use crate::types::*;
use crate::utf8::is_continuation_byte;

/// Parse block elements and then parse inline elementst within them.
pub fn parse_document(input: &str) -> Document {
  let block_elements = parse_block_elements(input);
  let inline_elements = parse_inline_elements(input, &block_elements);

  Document { block_elements, inline_elements }
}

pub fn parse_block_elements(input: &str) -> Vec<BlockElement> {
  let mut block_parser = BlockParser::new(input);
  block_parser.parse();
  block_parser.blocks
}

pub const fn parse_inline_elements(_input: &str, _block_elements: &[BlockElement]) -> Vec<InlineElement> {
  vec![/* todo */]
}

type BlockIndex = usize;

/// Parser that splits input text into block elements (first phase).
///
/// The [parsing strategy](https://spec.commonmark.org/0.30/#phase-1-block-structure) outlined in the CommonMark
/// specification assumes block elements are stored as a tree-like structure. Instead we're storing a flat vector of\
/// blocks and a separate stack of indicies of currently open blocks, because of the following reasons:
///
/// 1. Our parser doesn't need to produce a tree-like block structure.
/// 2. A vector has better cache locality than a tree-like structure.
///
/// Otherwise, the strategy is the same as outlined in the specification.
///
/// # Examples
///
/// ```rust
/// #![feature(assert_matches)]
///
/// use std::assert_matches::assert_matches;
///
/// use macaroni::*;
///
/// let input = "Hello, [world](https://en.wikipedia.org/wiki/World)!";
///
/// let mut block_parser = BlockParser::new(input);
/// let block_elements = block_parser.parse();
///
/// assert_eq!(block_elements.len(), 2);
/// assert_matches!(&block_elements[0], BlockElement::Root);
/// assert_matches!(&block_elements[1], BlockElement::Paragraph { .. });
/// ```
pub struct BlockParser<'a> {
  input: &'a str,

  offset: usize,
  character: usize,
  column: usize,
  line: usize,

  indent: usize,

  tab_leftovers: usize,

  blocks: Vec<BlockElement>,
  open_blocks: Vec<BlockIndex>,
}

impl<'a> BlockParser<'a> {
  #[must_use]
  pub fn new(input: &'a str) -> Self {
    Self {
      input,

      offset: 0,
      character: 0,
      column: 0,
      line: 0,

      indent: 0,
      tab_leftovers: 0,

      blocks: vec![BlockElement::Root],
      open_blocks: vec![0],
    }
  }

  pub fn parse(&mut self) -> &[BlockElement] {
    while self.offset < self.input.as_bytes().len() {
      self.parse_line();
    }
    &self.blocks
  }

  /// Process a line of input.
  ///
  /// Strategy outline:
  ///
  /// 1. Iterate over currenly open blocks and find all that "match", which means that their conditions for remaining
  /// open are met.
  /// 2. If last match is a container (or a paragaph, since paragraphs can be closed immediately), try to find new block
  /// starts.
  /// 3. If new block start is found, close all blocks after the last match and then append the found block; return to
  /// step, using the appended block intead of the last match.
  /// 4. If no block starts were found, the last match is a paragraph, and current line is not empty, add current line
  /// to the paragarph as a continuation line.
  /// 5. The rest of the line is either a paragraph or a text content of a previously created block.
  ///
  /// For more details to CommonMark specification (https://spec.commonmark.org/0.30/#phase-1-block-structure).
  fn parse_line(&mut self) {
    let last_match_open_index = self.last_match();

    if !self.parse_block(last_match_open_index) && !self.parse_continuation_line() {
      self.close_children_of(last_match_open_index);
    }

    let line_end = self.peek_line();

    let tip = &mut self.blocks[*self.open_blocks.last().unwrap()];

    match tip {
      BlockElement::Paragraph { .. } => {}

      BlockElement::AtxHeading { content_range } => {
        let bytes = self.input.as_bytes();
        let mut content_end = line_end;

        while bytes[content_end.offset - 1] == b'#' {
          content_end.offset -= 1;
          content_end.character -= 1;
        }

        while let b' ' | b'\t' = bytes[content_end.offset - 1] {
          content_end.offset -= 1;
          content_end.character -= 1;
        }

        content_range.end = content_end;

        self.set_position(line_end);
      }

      BlockElement::SetextHeading { .. } => {
        // add text content.
        todo!()
      }

      BlockElement::FencedCodeBlock | BlockElement::IndentedCodeBlock => {
        // do nothing
        // todo!()
        self.consume_line();
      }

      BlockElement::Root | BlockElement::BlockQuote => {
        if !self.is_at_line_end() {
          let start = self.position();
          self.consume_line();
          let end = self.position();

          self.append_child(BlockElement::Paragraph { lines: vec![Range { start, end }] });
        }
      }
    }

    self.consume_line_end();
  }

  fn last_match(&mut self) -> usize {
    let mut block_open_index = 0;
    while block_open_index < self.open_blocks.len() {
      self.consume_spaces();

      let block_index = self.open_blocks[block_open_index];
      let block = &self.blocks[block_index];

      let matches = match block {
        BlockElement::Root => true,
        BlockElement::BlockQuote => {
          if self.peek() == Some(b'>') && !self.is_indented() {
            self.consume_columns(1);
            if let Some(b' ' | b'\t') = self.peek() {
              self.consume_columns(1);
            }
            true
          } else {
            false
          }
        }
        BlockElement::Paragraph { .. } => !self.is_at_line_end(),
        BlockElement::AtxHeading { .. } => false,
        BlockElement::SetextHeading { .. } => todo!(),
        BlockElement::FencedCodeBlock => todo!(),
        BlockElement::IndentedCodeBlock => self.is_indented() || self.is_at_line_end(),
      };

      if !matches {
        return block_open_index - 1;
      }

      block_open_index += 1;
    }

    self.open_blocks.len() - 1
  }

  fn parse_block(&mut self, mut block_open_index: usize) -> bool {
    let mut result = false;
    let mut block_index = self.open_blocks[block_open_index];

    let mut is_paragraph = matches!(self.blocks[block_index], BlockElement::Paragraph { .. });

    while self.blocks[block_index].is_container() || is_paragraph {
      self.consume_spaces();
      let new_block = self.block_start_start();

      match new_block {
        Some(new_block) => {
          if is_paragraph {
            block_open_index -= 1;
            is_paragraph = false;
          }

          self.insert_child(block_open_index, new_block);
          block_open_index = self.open_blocks.len() - 1;
          block_index = self.open_blocks[block_open_index];
          result = true;
          self.indent = 0;
        }
        None => {
          break;
        }
      }
    }

    result
  }

  fn block_start_start(&mut self) -> Option<BlockElement> {
    or_else! {
      self.parse_block_quote_start(),
      self.parse_atx_heading_start(),
      self.parse_indented_code_block_start()
    }
  }

  fn parse_block_quote_start(&mut self) -> Option<BlockElement> {
    if !self.is_indented() && self.peek() == Some(b'>') {
      self.offset += 1;
      self.character += 1;
      self.tab_leftovers = 0;

      if let Some(b' ' | b'\t') = self.peek() {
        self.consume_columns(1);
      }
      Some(BlockElement::BlockQuote)
    } else {
      None
    }
  }

  fn parse_atx_heading_start(&mut self) -> Option<BlockElement> {
    if !self.is_indented() && self.peek() == Some(b'#') {
      let level = self.consume_hashes();

      if level <= 6 && let Some(b' ' | b'\t') = self.peek() {
        self.consume_spaces();
        let position = self.position();
        Some(BlockElement::AtxHeading { content_range: Range { start: position, end: position } })
      } else {
        // Restore previous position.
        // TODO: Restoring position can be moved to a method.

        self.offset -= level;
        self.character -= level;
        None
      }
    } else {
      None
    }
  }

  fn parse_indented_code_block_start(&mut self) -> Option<BlockElement> {
    let tip = &self.blocks[*self.open_blocks.last().unwrap()];
    if !matches!(tip, BlockElement::Paragraph { .. }) && self.is_indented() && !self.is_at_line_end() {
      Some(BlockElement::IndentedCodeBlock)
    } else {
      None
    }
  }

  fn parse_continuation_line(&mut self) -> bool {
    let is_at_line_end = self.is_at_line_end();
    let &tip_index = self.open_blocks.last().unwrap();

    let start = self.position();
    let end = self.peek_line();

    if let BlockElement::Paragraph { lines } = &mut self.blocks[tip_index] && !is_at_line_end {
      lines.push(Range { start, end });
      self.set_position(end);
      true
    } else {
      false
    }
  }

  fn close_children_of(&mut self, parent_open_index: usize) {
    debug_assert!(parent_open_index < self.open_blocks.len(), "Parent index is out of bounds.");
    self.open_blocks.truncate(parent_open_index + 1);
  }

  fn insert_child(&mut self, parent_open_index: usize, child: BlockElement) {
    self.close_children_of(parent_open_index);
    self.append_child(child);
  }

  fn append_child(&mut self, child: BlockElement) {
    debug_assert!(
      self.blocks[*self.open_blocks.last().unwrap()].is_container(),
      "Attempting to append a child to a leaf block."
    );

    self.open_blocks.push(self.blocks.len());
    self.blocks.push(child);
  }

  #[inline]
  const fn peek(&self) -> Option<u8> {
    if self.offset < self.input.as_bytes().len() {
      Some(self.input.as_bytes()[self.offset])
    } else {
      None
    }
  }

  fn consume_columns(&mut self, mut count: usize) {
    while count > 0 {
      if self.tab_leftovers > 0 {
        let columns_to_consume = count.min(self.tab_leftovers);
        count -= columns_to_consume;
        self.tab_leftovers -= columns_to_consume;
        if self.tab_leftovers == 0 {
          self.offset += 1;
          self.character += 1;
        }
      } else {
        match self.peek() {
          Some(b'\t') => {
            self.tab_leftovers = 4 - (self.column % 4);
          }
          Some(b) => {
            self.offset += 1;
            if !is_continuation_byte(b) {
              self.character += 1;
              self.column += 1;
              count -= 1;
            }
          }
          None => {
            return;
          }
        }
      }
    }
  }

  fn consume_spaces(&mut self) {
    self.tab_leftovers = 0;

    let bytes = self.input.as_bytes();
    let old_column = self.column;

    while self.offset < bytes.len() {
      let byte = bytes[self.offset];
      match byte {
        b' ' => {
          self.offset += 1;
          self.character += 1;
          self.column += 1;
        }
        b'\t' => {
          self.offset += 1;
          self.character += 1;
          self.column += 4;
        }
        _ => {
          break;
        }
      }
    }

    self.indent += self.column - old_column;
  }

  fn consume_hashes(&mut self) -> usize {
    self.tab_leftovers = 0;
    let old_offset = self.offset;

    while self.peek() == Some(b'#') {
      self.offset += 1;
      self.character += 1;
    }

    self.offset - old_offset
  }

  const fn peek_line(&self) -> Position {
    let bytes = self.input.as_bytes();
    let mut position = self.position();
    while position.offset < bytes.len() {
      let b = bytes[position.offset];
      match b {
        b'\n' | b'\r' => {
          return position;
        }
        _ => {
          position.offset += 1;
          if !is_continuation_byte(b) {
            position.character += 1;
          }
        }
      }
    }
    position
  }

  fn consume_line(&mut self) {
    self.tab_leftovers = 0;

    while let Some(b) = self.peek() {
      match b {
        b'\n' | b'\r' => {
          return;
        }
        _ => {
          self.offset += 1;
          if !is_continuation_byte(b) {
            self.character += 1;
          }
        }
      }
    }
  }

  fn consume_line_end(&mut self) {
    self.tab_leftovers = 0;
    self.indent = 0;
    let b = self.peek();
    match b {
      Some(b'\n' | b'\r') => {
        self.offset += 1;
        self.line += 1;
        self.character = 0;

        if b == Some(b'\r') && self.peek() == Some(b'\n') {
          self.offset += 1;
        }
      }
      Some(b) => {
        panic!("Expected new line or end of input, instead got {:?}.", char::from(b));
      }
      None => {}
    }
  }

  #[inline]
  const fn position(&self) -> Position {
    Position { line: self.line, character: self.character, offset: self.offset }
  }

  #[inline]
  fn set_position(&mut self, position: Position) {
    self.line = position.line;
    self.character = position.character;
    self.offset = position.offset;
  }

  #[inline]
  const fn is_indented(&self) -> bool {
    const CODE_INDENT: usize = 4;

    self.indent >= CODE_INDENT
  }

  #[inline]
  const fn is_at_line_end(&self) -> bool {
    matches!(self.peek(), Some(b'\n' | b'\r') | None)
  }
}

#[allow(dead_code)]
pub struct InlineParser<'a> {
  input: &'a str,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn paragraph_test() {
    let block_elements = parse_block_elements("foo\n    bar\n\nbaz");
    assert_eq!(block_elements.len(), 3); // Root, paragaph (with lazy continuation), paragraph.)
  }

  #[test]
  fn block_quote_test() {
    let block_elements = parse_block_elements("> foo\nbar\n>baz");
    assert_eq!(block_elements.len(), 3); // Root, block quote, paragraph.
  }

  #[test]
  fn nested_block_quote_test() {
    let block_elements = parse_block_elements("> foo\n> > bar");
    assert_eq!(block_elements.len(), 5);
  }
}

#[cfg(bar)]
fn foo() {
  let x = vec![
    Root,
    BlockQuote,
    Paragraph {
      lines: [
        Range {
          start: Position { line: 0, character: 2, offset: 2 },
          end: Position { line: 0, character: 5, offset: 5 },
        },
        Range {
          start: Position { line: 1, character: 2, offset: 8 },
          end: Position { line: 1, character: 7, offset: 13 },
        },
      ],
    },
  ];
}

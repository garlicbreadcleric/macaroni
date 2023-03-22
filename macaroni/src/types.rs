use serde::Serialize;

#[derive(Copy, Clone, Debug, Default, Serialize)]
pub struct Position {
  pub line: usize,
  pub character: usize,
  pub offset: usize,
}

#[derive(Copy, Clone, Debug, Serialize)]
pub struct Range {
  pub start: Position,
  pub end: Position,
}

#[derive(Copy, Clone, Debug, Serialize)]
pub struct HeadingLevel(u8);

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
  pub block_elements: Vec<BlockElement>,
  pub inline_elements: Vec<InlineElement>,
}

/// Structural element that can contain other blocks or inline content.
///
/// There are two types of blocks: leaf blocks (<https://spec.commonmark.org/0.30/#leaf-blocks>) and container blocks
/// (<https://spec.commonmark.org/0.30/#container-blocks>). Unlike leaf blocks, container blocks can contain other
/// blocks.
///
/// Container blocks:
///
/// - [Block quote](BlockElement::BlockQuote)
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum BlockElement {
  /// Document root.
  ///
  /// Can only present once in the element list.
  Root,

  /// Block quote.
  ///
  /// # Examples
  ///
  /// ```markdown
  /// > block quote
  /// ```
  BlockQuote,

  /// Paragraph.
  ///
  /// # Examples
  ///
  /// ```markdown
  /// paragraph
  /// ```
  Paragraph { lines: Vec<Range> },

  /// Atx heading.
  ///
  /// # Examples
  ///
  /// ```markdown
  /// # heading 1
  /// ## heading 2
  /// ### heading 3 ##
  /// ```
  #[serde(rename_all = "camelCase")]
  AtxHeading { content_range: Range },

  /// Setext heading.
  ///
  /// See <https://spec.commonmark.org/0.30/#setext-headings>.
  ///
  /// # Examples
  ///
  /// ```markdown
  /// heading 1
  /// =========
  ///
  /// heading 2
  /// ---------
  ///
  /// multi-line
  /// heading 2
  /// ---------
  /// ```
  #[serde(rename_all = "camelCase")]
  SetextHeading { level: HeadingLevel, content_range: Range },

  /// Fenced code block.
  ///
  /// # Examples
  ///
  /// ~~~markdown
  /// ```
  /// code block
  /// ```
  ///
  /// ```rust
  /// code block
  /// ```
  ///
  /// ```{.rust}
  /// code block
  /// ```
  /// ~~~
  FencedCodeBlock,

  /// Indented code block.
  ///
  /// # Examples
  ///
  /// ```markdown
  ///     code block
  /// ```
  IndentedCodeBlock,
}

impl BlockElement {
  pub const fn is_container(&self) -> bool {
    match self {
      Self::Root | Self::BlockQuote => true,

      Self::Paragraph { .. }
      | Self::AtxHeading { .. }
      | Self::SetextHeading { .. }
      | Self::FencedCodeBlock
      | Self::IndentedCodeBlock => false,
    }
  }

  pub const fn is_leaf(&self) -> bool {
    !self.is_container()
  }
}

/// Inline content, such as raw text, a link, a code span etc.
///
/// Some inline elements can contain other elements, but an
/// inline element cannot contain a block element.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum InlineElement {
  /// Inline link.
  ///
  /// See <https://spec.commonmark.org/0.30/#inline-link>.
  ///
  /// # Examples
  ///
  /// ```markdown
  /// [text](destination)
  /// [text](<destination>)
  /// [text](destination "title")
  /// [text](destination 'title')
  /// [text](destination (title))
  /// ```
  #[serde(rename_all = "camelCase")]
  InlineLink { text_range: Range, destination_range: Range, title_range: Option<Range> },

  /// Reference link.
  ///
  /// See <https://spec.commonmark.org/0.30/#reference-link>.
  ///
  /// # Examples
  ///
  /// ```markdown
  /// [text][reference]
  /// [reference][]
  /// [reference]
  /// ```
  ReferenceLink {/* text: Option<WithRange<ReferenceLinkText>>, reference: WithRange<Reference> */},

  /// Inline code span.
  ///
  /// See <https://spec.commonmark.org/0.30/#code-span>.
  ///
  /// ```markdown
  /// `code`
  /// `` co ` de ``
  /// ```
  CodeSpan,

  /// Raw text.
  ///
  /// Text elements also include inlines that Macaroni ignores, like emphasis or
  /// strong emphasis.
  ///
  /// # Examples
  ///
  /// ```markdown
  /// text
  /// text with _emphasis_ and **strong emphasis**
  /// ```
  Text,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceLinkText {
  content_range: Range,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reference {
  content_range: Range,
}

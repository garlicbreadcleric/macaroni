export type Position = { line: number; character: number; offset: number };

export type Range = { start: Position; end: Position };

export type BlockElement =
  | { type: "root" }
  | { type: "paragraph"; lines: Range[] }
  | { type: "blockQuote" }
  | { type: "atxHeading"; contentRange: Range };

export type InlineElement = { type: "text"; range: Range };

export type Document = { blockElements: BlockElement[]; inlineElements: InlineElement[] };

export type EditorState = {
  source: string;
};

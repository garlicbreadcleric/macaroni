# Macaroni (WIP)

Markdown parser for language servers.

## Goals

- CommonMark with some Pandoc Markdown syntax extetnsions (citations, footnotes).
- Character positions based on UTF-16 code points. There are different character counting modes that language servers/clients _can_ support, but UTF-16 based is the only one that all servers/clients _must_ support.
- Performance. Some SIMD optimizations are planned, as well as using third-party SIMD-optimized libraries where possible.

## License

GNU LGPLv3 (c) Daniil Kolesnichenko

import React, { useState, useMemo, useEffect } from "https://esm.sh/react@18.2.0?dev";
import ReactDOM from "https://esm.sh/react-dom@18.2.0/client?dev";
import { useQuery, QueryClient, QueryClientProvider } from "https://esm.sh/@tanstack/react-query@4.27.0?dev";

import { Button, MantineProvider, Flex, Title } from "https://esm.sh/@mantine/core@6.0.2?dev";
import { IconFileExport, IconFileImport } from "https://esm.sh/@tabler/icons-react@2.11.0?dev";

import "https://esm.sh/ace-builds@1.16.0/src-min-noconflict/ace.js";
import "https://esm.sh/ace-builds@1.16.0/src-min-noconflict/mode-plain_text?dev";
import "https://esm.sh/ace-builds@1.16.0/src-min-noconflict/mode-yaml?dev";
import "https://esm.sh/ace-builds@1.16.0/src-min-noconflict/theme-twilight?dev";
import "https://esm.sh/ace-builds@1.16.0/src-min-noconflict/theme-solarized_dark?dev";
import AceEditor, { IAceEditorProps, IMarker } from "https://esm.sh/react-ace@10.1.0?dev";

import YAML from "https://esm.sh/yaml@2.2.1?dev";
import Color from "https://esm.sh/color?dev";

import type { Document, EditorState } from "./types.ts";

type HeaderProps = any;

function Header(props: HeaderProps) {
  return (
    <Flex align="center" justify="space-between" p="xs" gap="xs" bg="#01313f">
      <Title order={2} style={{ cursor: "default", userSelect: "none" }} ml="sm">
        Macaroni Web UI
      </Title>
      <Flex gap="xs">
        <Button color="lime" variant="light" leftIcon={<IconFileImport size="1rem" />}>
          Import
        </Button>
        <Button color="lime" variant="light" leftIcon={<IconFileExport size="1rem" />}>
          Export
        </Button>
      </Flex>
    </Flex>
  );
}

type EditorProps = IAceEditorProps;

function Editor(props: EditorProps) {
  return (
    <AceEditor
      mode="plain_text"
      theme="solarized_dark"
      width="100%"
      height="100%"
      fontSize="14px"
      tabSize={2}
      showPrintMargin={false}
      highlightActiveLine={false}
      {...props}
    />
  );
}

function getInitialState() {
  const stored = localStorage.getItem("state");
  if (stored !== null) {
    return JSON.parse(stored);
  } else {
    return {
      source: "Hello, [world](https://en.wikipedia.org/wiki/World)!",
    };
  }
}

function saveState(state: EditorState) {
  localStorage.setItem("state", JSON.stringify(state));
}

function rangesToMarkers({ inlineElements, blockElements }: Document): IMarker[] {
  const inlineMarkers =
    inlineElements.flatMap((e): IMarker[] => {
      switch (e.type) {
        case "text":
          return [
            {
              startRow: e.range.start.line,
              startCol: e.range.start.character,
              endRow: e.range.end.line,
              endCol: e.range.end.character,
              className: `parsed-inline-${e.type.toLowerCase()}`,
              type: "text",
              // type: 'fullLine'
            },
          ];
        default:
          return [];
      }
    }) ?? [];

  const blockMarkers =
    blockElements.flatMap((e): IMarker[] => {
      switch (e.type) {
        case "paragraph":
          return e.lines.flatMap((r) => ({
            startRow: r.start.line,
            startCol: r.start.character,
            endRow: r.end.line,
            endCol: r.end.character,
            className: `parsed-block-${e.type}`,
            type: "text",
          }));
        case "atxHeading":
          return [
            {
              startRow: e.contentRange.start.line,
              startCol: e.contentRange.start.character,
              endRow: e.contentRange.end.line,
              endCol: e.contentRange.end.character,
              className: `parsed-block-${e.type}`,
              type: "text",
            },
          ];
        default:
          return [];
      }
    }) ?? [];

  return [...inlineMarkers, ...blockMarkers];
}

function App() {
  const initialState = useMemo(getInitialState, []);
  const [source, setSource] = useState(initialState.source);

  const { data }: { data?: Document } = useQuery([source], () => {
    return fetch("/parse", {
      method: "POST",
      headers: {
        ["content-type"]: "application/json",
      },
      body: JSON.stringify({
        source,
      }),
    }).then((r) => r.json());
  });

  useEffect(() => saveState({ source }), [source]);

  return (
    <Flex direction="column" style={{ width: "100vw", height: "100vh" }}>
      <Header />
      <Flex
        style={{ width: "100%", height: "100%", backgroundColor: new Color("#002b36").darken(0.1).hex() }}
        gap="xs"
        p="xs"
      >
        <Editor
          width="120%"
          value={source}
          onChange={setSource}
          focus
          markers={rangesToMarkers(data ?? { blockElements: [], inlineElements: [] })}
          wrapEnabled={false}
          showGutter={false}
        />
        <Editor
          scrollMargin={[10]}
          value={YAML.stringify(data)}
          mode="yaml"
          readOnly
          showGutter={false}
          wrapEnabled={false}
        />
      </Flex>
    </Flex>
  );
}

const queryClient = new QueryClient();

const root = ReactDOM.createRoot(document.querySelector("#root")!);
root.render(
  <QueryClientProvider client={queryClient}>
    <MantineProvider theme={{ colorScheme: "dark" }} withGlobalStyles withNormalizeCSS>
      <App />
    </MantineProvider>
  </QueryClientProvider>
);

---
description: Invariants that all parsers must maintain
---
Every parser in `src/parse/` must uphold these invariants:

1. **Never panic on malformed input.** Return a `Result<T, String>` error or skip the malformed item with a warning via `crate::log::warn()`. Users will feed this tool arbitrary markdown.

2. **Frontmatter is consumed, never forwarded.** The `---` delimited frontmatter block and its contents must not appear in the `body` field of the resulting IR struct. The body is what gets written to the target file.

3. **Target overrides use the `target: key=value` format.** The target name (e.g. `cursor`) is the HashMap key. The key=value pair is split on the first `=` only — values may contain `=`.

4. **Parsers are pure functions of their input.** They read files but have no side effects. Logging warnings is the one exception (for skipped/malformed items).

---
description: Contract that all generators must follow
---
Every generator in `src/generate/` must follow this contract:

1. **Input is `&ProjectConfig` (or a subset), output is `Vec<GeneratedFile>`.** Generators never touch the filesystem. They produce a list of path + content pairs. The `output` module handles writing.

2. **Every generated file must include the `generated-by: agentic-sync` marker.** For YAML frontmatter files (.mdc), this is a frontmatter field. For other files (e.g. copilot-instructions.md), use an HTML comment.

3. **Cursor overrides from `target_overrides.get("cursor")` replace defaults, they don't merge.** If the user sets `cursor: alwaysApply=false`, the default `true` is replaced, not appended.

4. **Generators are stateless.** No caching, no reading previous output, no side effects. Every run produces the complete set of files from scratch.

5. **Paths are absolute**, constructed from the `root` parameter. The output module handles relative display.

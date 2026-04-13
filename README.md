# agentic.md

Write your AI coding instructions once. Use them everywhere.

`agentic-sync` reads your Claude Code configuration — `CLAUDE.md`, rules, skills, and MCP servers — and generates native config files for Cursor, GitHub Copilot, and more. Claude is the source of truth. Everything else stays in sync.

```
agentic-sync --fix
```

```
CLAUDE.md                        .cursor/rules/stack.mdc
                          -->    .cursor/rules/testing.mdc
                                 .cursor/rules/conventions.mdc
                                 .cursor/skills/explain-code.mdc
                                 .cursor/mcp.json
                                 .github/copilot-instructions.md
```

## Why

Every AI coding tool has its own config format. Cursor uses `.mdc` files with YAML frontmatter. Copilot uses `.github/copilot-instructions.md`. Claude uses `CLAUDE.md` and `.claude/skills/`. If you use more than one tool — or your team does — you maintain the same rules in multiple places.

`agentic-sync` eliminates that. Write your rules in Claude's format, run the tool, and every other tool gets native config files generated from the same source.

## Install

### From GitHub Releases

Download the latest binary for your platform from [Releases](https://github.com/NorthIsUp/agentic.md/releases):

| Platform | Binary |
|---|---|
| macOS (Apple Silicon) | `agentic-sync-aarch64-darwin` |
| macOS (Intel) | `agentic-sync-x86_64-darwin` |
| Linux (x86_64) | `agentic-sync-x86_64-linux` |
| Linux (aarch64) | `agentic-sync-aarch64-linux` |
| Windows (x86_64) | `agentic-sync-x86_64-windows.exe` |

```sh
# Example: macOS Apple Silicon
curl -L https://github.com/NorthIsUp/agentic.md/releases/latest/download/agentic-sync-aarch64-darwin -o agentic-sync
chmod +x agentic-sync
mv agentic-sync /usr/local/bin/
```

### From Source

```sh
cargo install --git https://github.com/NorthIsUp/agentic.md
```

## Quick Start

If you already have a `CLAUDE.md` in your project:

```sh
# Generate Cursor + Copilot config
agentic-sync --fix

# Verify everything is in sync
agentic-sync --check
```

That's it. Commit the generated files so teammates using Cursor or Copilot get the same rules.

## What Gets Synced

### CLAUDE.md &rarr; Cursor Rules

Each `## Heading` in your `CLAUDE.md` becomes a separate `.cursor/rules/{slug}.mdc` file with `alwaysApply: true`.

```markdown
<!-- CLAUDE.md -->

# My Project

General project context goes here.

## Stack
TypeScript, React, Tailwind

## Testing
Run tests with `npm test`. Always write tests first.

## Conventions
Use conventional commits. No default exports.
```

Generates:

```
.cursor/rules/_project.mdc       # "General project context goes here."
.cursor/rules/stack.mdc           # "TypeScript, React, Tailwind"
.cursor/rules/testing.mdc         # "Run tests with..."
.cursor/rules/conventions.mdc     # "Use conventional commits..."
```

### CLAUDE.md &rarr; Copilot Instructions

The full `CLAUDE.md` is copied verbatim to `.github/copilot-instructions.md`. Copilot uses one flat file, so no splitting is needed.

### .claude/rules/*.md &rarr; Cursor Rules

Additional rule files in `.claude/rules/` are synced as individual `.cursor/rules/{name}.mdc` files.

### .claude/skills/*/SKILL.md &rarr; Cursor Skills

Claude skills with frontmatter are mapped to Cursor's `.mdc` format:

| Claude field | Cursor field |
|---|---|
| `name` | filename |
| `description` | `description` |
| `paths` | `globs` |
| `disable-model-invocation: true` | `alwaysApply: false` |

### .mcp.json &rarr; .cursor/mcp.json

MCP server configs are transformed to Cursor's expected schema. Environment variable interpolation (`${VAR}`) passes through as-is.

## Cursor-Specific Overrides

Need different behavior in Cursor than in Claude? Add `cursor:` lines in a frontmatter block within your `CLAUDE.md` section:

```markdown
## Testing
---
cursor: alwaysApply=false
cursor: globs=**/*.test.ts
---
Run tests with `npm test`. Always write tests first.
```

This sets `alwaysApply: false` and `globs: **/*.test.ts` in the generated `.cursor/rules/testing.mdc`, while Claude sees the frontmatter as inert text and ignores it. Each `cursor:` line maps directly to a Cursor frontmatter field.

The same pattern works in `.claude/skills/*/SKILL.md` — just add `cursor:` lines alongside the standard Claude frontmatter.

## Ownership & Safety

Generated files include a `generated-by: agentic-sync` marker in their frontmatter. This protects hand-written files:

- **`--fix`** skips existing files that don't have the marker (warns instead)
- **`--fix --overwrite`** writes regardless
- **Cleanup** deletes generated files whose source was removed, but never touches hand-written files

You can safely mix generated and hand-written Cursor rules in the same directory.

## CLI Reference

```
agentic-sync [--check] [--fix] [--pr] [--out=<targets>] [--overwrite] [path]
```

| Flag | Description |
|---|---|
| `--check` | Compare generated files to disk. Exit 1 if stale. **(default)** |
| `--fix` | Write generated files to disk. |
| `--pr` | Output a markdown diff summary to stdout (for PR comments). |
| `--out=<targets>` | Targets to generate: `cursor`, `copilot`. Comma-separated or repeated. Default: all. |
| `--overwrite` | With `--fix`: overwrite files even without `generated-by` marker. |
| `path` | Project root. Defaults to current directory. |

### Exit Codes

| Code | Meaning |
|---|---|
| 0 | In sync (check) or success (fix) |
| 1 | Stale files (check) or skipped files (fix) |
| 2 | Parse error |

### CI / GitHub Actions

When `GITHUB_ACTIONS=true` is set, output uses [GitHub annotations](https://docs.github.com/en/actions/writing-workflows/choosing-what-your-workflow-does/workflow-commands-for-github-actions):

```yaml
- run: agentic-sync --check
```

Stale files appear as warnings in the PR diff.

## Git Hook Integration

### With [hk](https://github.com/jdx/hk)

```pkl
["agentic-sync"] {
    glob = List("CLAUDE.md", ".claude/**", ".mcp.json")
    check = "agentic-sync --check"
    fix = "agentic-sync --fix"
}
```

### With pre-commit (manual)

```sh
# .git/hooks/pre-commit
agentic-sync --check || { echo "Run 'agentic-sync --fix' to sync"; exit 1; }
```

## Supported Targets

| Target | Status | Output |
|---|---|---|
| Cursor | Supported | `.cursor/rules/*.mdc`, `.cursor/skills/*.mdc`, `.cursor/mcp.json` |
| GitHub Copilot | Supported | `.github/copilot-instructions.md` |
| Codex | Planned | `AGENTS.md` |
| Gemini | Planned | `GEMINI.md` |

## How It Works

```
 Source (Claude)              Intermediate            Targets
                              Representation

 CLAUDE.md ──────┐                              ┌──> .cursor/rules/*.mdc
                 │                              │
 .claude/rules/  ├──> parse ──> ProjectConfig ──├──> .cursor/skills/*.mdc
                 │                              │
 .claude/skills/ ┤                              ├──> .cursor/mcp.json
                 │                              │
 .mcp.json ──────┘                              └──> .github/copilot-instructions.md
```

Sources are parsed into a common `ProjectConfig` struct, then each target generator transforms that IR into native config files. Adding a new target means writing one generator — the parsers don't change.

## License

MIT

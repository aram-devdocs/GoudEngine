---
name: humanizer
description: Remove AI writing patterns from documentation, README, and comments
user-invocable: true
---

# Humanizer

Detect and rewrite AI-sounding prose in documentation, README files, code comments, and any user-facing text.

## When to Use

Run on all documentation changes before committing. Mandatory for README.md updates, CLAUDE.md files, doc comments, and any prose that will be read by humans.

## 6 Anti-Pattern Categories

### 1. Filler Adverbs and Intensifiers

Remove or replace words that add no information:

| Remove | Replace With |
|--------|-------------|
| "effectively" | (delete) |
| "efficiently" | (delete or be specific: "in O(n) time") |
| "seamlessly" | (delete) |
| "significantly" | (be specific: "by 40%") |
| "extremely" | (delete) |
| "highly" | (delete) |
| "incredibly" | (delete) |

### 2. Corporate/Marketing Buzzwords

| Avoid | Use Instead |
|-------|-------------|
| "leverage" | "use" |
| "utilize" | "use" |
| "comprehensive" | "full" or "complete" |
| "robust" | "reliable" or describe what makes it so |
| "scalable" | describe the actual scaling property |
| "cutting-edge" | (delete or describe the specific advancement) |
| "state-of-the-art" | (delete or cite what's novel) |
| "innovative" | (delete or describe the innovation) |
| "empower" | "let" or "allow" |
| "streamline" | "simplify" |

### 3. Hedging and Weasel Words

| Avoid | Use Instead |
|-------|-------------|
| "It's worth noting that" | (just state the thing) |
| "It should be noted" | (just state the thing) |
| "In order to" | "To" |
| "Due to the fact that" | "Because" |
| "At the end of the day" | (delete) |
| "A wide range of" | (be specific) |
| "In terms of" | (rephrase directly) |

### 4. AI Structural Patterns

- Avoid starting paragraphs with "This" referring to the project
- Avoid "Let's dive into..." or "Let's explore..."
- Avoid numbered lists where a single sentence works
- Avoid repeating the heading content in the first sentence
- Avoid concluding sections with summary sentences that restate what was just said

### 5. Exclamation and Enthusiasm

- Remove exclamation marks from technical writing
- Remove "!" from headings
- Replace "Great news!" style openings with direct statements
- Technical docs should be calm and factual

### 6. Passive Voice Overuse

| Passive | Active |
|---------|--------|
| "The component is rendered by the engine" | "The engine renders the component" |
| "Errors are handled by the error module" | "The error module handles errors" |
| "Tests can be run using cargo" | "Run tests with `cargo test`" |

## Rewrite Process

1. **Read** the target text
2. **Scan** for each of the 6 anti-pattern categories
3. **Rewrite** flagged sections:
   - Keep the same meaning
   - Use direct, plain language
   - Prefer short sentences
   - Use active voice
   - Be specific instead of vague
4. **Verify** the rewrite preserves technical accuracy
5. **Compare** before/after to confirm improvement

## Examples

**Before** (AI-sounding):
> GoudEngine provides a comprehensive and robust game engine solution that leverages Rust's powerful type system to deliver incredibly efficient rendering capabilities. It seamlessly integrates with multiple SDK targets.

**After** (human):
> GoudEngine is a Rust game engine with C#, Python, and native Rust SDKs. The type system catches errors at compile time, and the renderer batches draw calls to minimize GPU state changes.

**Before**:
> It's worth noting that the FFI layer effectively bridges the gap between Rust and the SDK languages.

**After**:
> The FFI layer connects Rust to the C# and Python SDKs.

## Scope

Apply to:
- `README.md`
- `CLAUDE.md` (root and subdirectory)
- `AGENTS.md`, `GEMINI.md`
- Doc comments (`///` in Rust, `///` XML docs in C#, docstrings in Python)
- PR descriptions
- Commit messages (light touch — just remove obvious AI patterns)

Do NOT apply to:
- Code (variable names, function names)
- Error messages (keep them concise and technical)
- Log messages
- Test names

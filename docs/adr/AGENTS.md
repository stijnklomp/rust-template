# AGENTS.md — ADRs

Scope: `docs/adr/`. The root `AGENTS.md` holds repo-wide rules; the closest `AGENTS.md` wins.

## Overview

ADRs record **why** a decision was made and what it rules out. One decision per file, numbered and append-only. The code and tests show how; the ADR preserves the trade-off. `docs/agents/domain.md` covers how to consume the corpus; this file covers writing to it.

## Setup & environment

- No tooling: plain Markdown in `docs/adr/`.
- Read `GLOSSARY.md` and the ADRs that touch the area before writing.
- An ADR records a decision that has been made; it is not a proposal.

## Conventions

- File name: zero-padded four-digit number + kebab-case title, e.g. `0013-migration-jobs.md`. The next number is the highest existing + 1 (`ls docs/adr`).
- Title: the decision, not the topic — "One binary per service; no runtime modes", not "Deployment".
- Body: a few paragraphs with context, the decision, and consequences or rejected alternatives. Match the existing files: no headings, no status field.
- Append-only: never rewrite an accepted ADR. Supersede it with a new one and link both ways (`Supersedes ADR-0008` / `Superseded by ADR-0013`).
- Link the code, tests, recipes and related ADRs that make the decision concrete.
- A vocabulary change also updates `GLOSSARY.md`.
- Update the nearest `AGENTS.md` only when the decision changes how an agent works; otherwise the ADR is the record.

## Common patterns

| Situation | Do |
| --- | --- |
| New decision | New file with the next number; link the code and related ADRs |
| Reversal | New ADR with `Supersedes ADR-00xx`; add `Superseded by ADR-00yy` to the old file |
| Vocabulary change | ADR + `GLOSSARY.md` update in the same change |
| Local, reversible choice | A code comment may be enough; an ADR is for constraints that outlive the change |

## Security & safety

- ADRs are committed history: no credentials, customer data, internal URLs or incident details.
- Describe threats and trade-offs abstractly and link private material instead of pasting it.

## Examples

- `0003-sqlx-and-migrations.md` — a concise decision plus the reasoning (agent-first, offline cache).
- `0011-executable-specs.md` and `0012-test-layout.md` — recent decisions that include rejected alternatives.
- Copy their shape: title, then a few paragraphs.

## Checklist

- [ ] Next number, zero-padded, kebab-case title.
- [ ] One decision with context, decision and consequences.
- [ ] Links to the code/tests/recipes and related ADRs.
- [ ] Supersession links added both ways, if any.
- [ ] `GLOSSARY.md` updated if vocabulary changed.
- [ ] No credentials, PII or internal URLs.

## When stuck

- Number collision → pull, re-check `ls docs/adr`, and renumber only your uncommitted file.
- Unsure whether it is an ADR → if the choice is local and reversible, a code comment or an `AGENTS.md` note is enough; if it constrains future work, write an ADR.
- The change contradicts an existing ADR → do not silently override; write the superseding ADR and flag it in the PR.
- Need the rationale for existing code → search `docs/adr/` for the module or term; `docs/agents/domain.md` gives the reading order.

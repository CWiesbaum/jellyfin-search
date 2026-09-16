<!--
Sync Impact Report
Version change: (unratified template) → 1.0.0
Rationale: Initial ratification. The prior file at this path was the unfilled
constitution-template scaffold (all placeholder tokens, no adopted content), so this is
treated as the project's first concrete constitution rather than an amendment.
Modified principles: none (initial set)
Added sections:
  - Core Principles: I. Test-First (NON-NEGOTIABLE), II. Simplicity & YAGNI,
    III. Code Review Discipline, IV. Observability, V. Semantic Versioning & Breaking Changes
  - Quality Standards (Section 2)
  - Development Workflow (Section 3)
  - Governance
Removed sections: none
Templates requiring updates:
  - .specify/templates/plan-template.md — ⚠ pending manual check (Constitution Check gate
    should reference these five principles)
  - .specify/templates/spec-template.md — ✅ no principle-specific references
  - .specify/templates/tasks-template.md — ✅ no principle-specific references
  - .specify/templates/checklist-template.md — ✅ no principle-specific references
Follow-up TODOs:
  - TODO(RATIFICATION_DATE): No prior ratification date exists in repo history; set to the
    date this version was adopted (2026-09-16). Confirm or correct if the project considers
    an earlier date authoritative.
-->

# Jellyfin Search Constitution

## Core Principles

### I. Test-First (NON-NEGOTIABLE)
Tests MUST be written before implementation code, MUST be reviewed and approved, and MUST
fail before the corresponding implementation is written. Development follows a strict
Red-Green-Refactor cycle: write a failing test, make it pass with the minimum necessary code,
then refactor. No feature or bug fix MAY be merged without tests that demonstrate the
behavior it claims to provide.
Rationale: Untested behavior is unverified behavior. Writing tests first forces requirements
to be explicit before code exists and prevents tests from being retrofitted to match
whatever the implementation happened to do.

### II. Simplicity & YAGNI
Every change MUST start with the simplest design that satisfies the current, stated
requirement. Abstractions, configuration options, and extensibility hooks MUST NOT be added
for hypothetical future needs; they are added only when a present requirement demands them.
Any added complexity (a new layer, pattern, or dependency) MUST be justified in the PR
description by the concrete requirement it serves.
Rationale: Speculative complexity is a maintenance cost paid immediately for a benefit that
may never arrive. Keeping the codebase minimal keeps it changeable.

### III. Code Review Discipline
All non-trivial changes MUST be reviewed by someone other than the author before merging to
a shared branch. Reviewers MUST verify the change against this constitution (tests-first
evidence, absence of unjustified complexity, observability, versioning impact) in addition to
correctness. Self-merging without review is reserved for trivial, low-risk changes (e.g.,
typo fixes, formatting).
Rationale: Review is the primary control point for catching constitutional drift before it
reaches shared branches, and it is cheaper to correct course before a merge than after.

### IV. Observability
Errors and significant state transitions MUST be surfaced through structured, readable
logging or equivalent diagnostic output, sufficient to diagnose a failure without attaching a
debugger. Interfaces SHOULD prefer text-based or otherwise inspectable I/O over opaque binary
formats where feasible, to keep behavior debuggable from logs and command-line tools alone.
Rationale: A system that cannot be observed cannot be operated confidently. Debuggability
from logs is what keeps incidents short.

### V. Semantic Versioning & Breaking Changes
Released artifacts (packages, APIs, CLIs) MUST follow MAJOR.MINOR.PATCH semantic versioning.
A breaking change to a public interface REQUIRES a MAJOR version increment and MUST be
accompanied by migration notes in the PR/commit description. New backward-compatible
functionality increments MINOR; fixes and clarifications increment PATCH.
Rationale: Predictable versioning lets consumers (including other parts of this project)
upgrade safely and lets the team reason about compatibility without re-reading every diff.

## Quality Standards

All code MUST pass automated linting and the full automated test suite before merge. Public
functions, modules, and interfaces MUST carry documentation sufficient for a caller to use
them without reading their implementation. Dead code, commented-out code, and unused
dependencies MUST NOT be merged into shared branches — if it is not used, it is removed, not
disabled.

## Development Workflow

Feature work follows the spec-kit flow: specify → plan → tasks → implement, with each stage's
artifacts kept under the feature's `specs/` directory so the rationale behind a change remains
traceable. Every pull request MUST reference the spec or task it implements. Direct commits to
`main` are discouraged in favor of reviewed pull requests, except for the trivial-change
exception described under Code Review Discipline.

## Governance

This constitution supersedes other informal practices where they conflict. Amendments are
made by editing this file, MUST state a rationale for the change, and MUST update the version
number according to the policy below:
- **MAJOR**: Backward-incompatible removal or redefinition of a principle or governance rule.
- **MINOR**: A new principle or materially expanded section is added.
- **PATCH**: Wording, clarification, or other non-semantic refinement.

All pull requests and reviews MUST verify compliance with this constitution; any deviation or
added complexity MUST be explicitly justified in the PR description. Runtime, day-to-day
development guidance that supplements but does not override this constitution belongs in
agent-specific guidance files (e.g., `CLAUDE.md`), not here.

**Version**: 1.0.0 | **Ratified**: 2026-09-16 | **Last Amended**: 2026-09-16

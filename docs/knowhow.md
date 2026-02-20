Great question. These are **design artifacts**, not code — and knowing when and how to use them is a _huge_ step toward building systems that scale in complexity and contributors.

I’ll explain **RFC**, **ADR**, when to use each, and then give you a **mental model + catalog** of the other ones worth knowing.

---

# 1. RFC (Request for Comments)

## What it is

An **RFC** is a **proposal for change or behavior** that is:

- Forward-looking
- Reviewable
- Meant to be discussed _before_ implementation

Originally from the IETF (e.g. HTTP, TCP), now widely used in software projects.

---

## What RFCs are used for

Use an RFC when you are deciding **what the system should do**.

RFCs answer questions like:

- What is this feature?
- Why does it exist?
- What problem does it solve?
- What guarantees does it provide?
- What are the trade-offs?
- What is _out of scope_?

---

## Typical RFC contents

- Problem statement
- Motivation
- Goals / non-goals
- Design
- Invariants
- Backward compatibility
- Alternatives considered
- Open questions

---

## When YOU should use an RFC

For Scarff, RFCs are ideal for:

✅ CLI command behavior (`scarff new`)
✅ Language / framework inference rules
✅ Template system design
✅ Plugin system
✅ Configuration model
✅ Release & versioning strategy

❌ Small refactors
❌ Implementation details

---

## Example RFC titles

- RFC-0001: `scarff new` Command Specification
- RFC-0002: Template Resolution & Overrides
- RFC-0003: Framework Inference Engine
- RFC-0004: Release & Distribution Strategy

---

## Mental model

> **RFC = “What are we building, and why?”**

---

# 2. ADR (Architecture Decision Record)

## What it is

An **ADR** records a **decision that has already been made**.

It is **backward-looking**, not a proposal.

---

## What ADRs are used for

ADRs answer:

- Why did we choose this?
- What alternatives were rejected?
- What constraints influenced the choice?
- What are the consequences?

They exist to prevent:

- Re-litigating old decisions
- Confusion for new contributors
- Architecture drift

---

## Typical ADR format

Most common (Michael Nygard format):

```md
# ADR-0007: Use Axum for Rust Web Backends

## Status

Accepted

## Context

We need an async web framework with strong type safety...

## Decision

We will use Axum.

## Consequences

- Pros
- Cons
```

---

## When YOU should use an ADR

For Scarff, ADRs are perfect for:

✅ Choosing Axum over Actix
✅ Using Rust for core instead of Go
✅ Using Cocogitto for versioning
✅ Branching strategy decisions
✅ CI/CD tooling decisions

❌ Features not yet designed
❌ Spec-level decisions

---

## Mental model

> **ADR = “Why does the system look like this?”**

---

# 3. RFC vs ADR (Side-by-side)

| Aspect          | RFC               | ADR                    |
| --------------- | ----------------- | ---------------------- |
| Time            | Before decision   | After decision         |
| Direction       | Forward-looking   | Backward-looking       |
| Purpose         | Propose & discuss | Record & explain       |
| Changes allowed | Yes               | No (unless superseded) |
| Tone            | Exploratory       | Definitive             |

---

# 4. Other Design Artifacts You Should Know

These are **less talked about**, but extremely useful.

---

## 4.1 PRD (Product Requirements Document)

**What:** User-facing requirements
**Who:** Product mindset (even solo devs)

Used to answer:

- Who is this for?
- What problem does it solve?
- What does success look like?

For Scarff:

- PRD for “MVP CLI Scaffolding Tool”

---

## 4.2 TDD / Test Plan (not Test-Driven Dev)

**What:** Explicit test strategy
**Purpose:** Prevents untestable designs

You already started this with your **test matrix** — that’s excellent.

---

## 4.3 Design Doc (Lightweight)

**What:** Less formal than RFC
**When:** Small or internal-only changes

Often used when:

- You don’t want full RFC ceremony
- Decision scope is limited

---

## 4.4 SPEC (Specification)

**What:** Precise, normative behavior description
**Tone:** “MUST / SHOULD / MAY”

Examples:

- CLI argument grammar
- Template metadata format
- Config file schema

RFCs often evolve into specs.

---

## 4.5 CHANGELOG

**What:** User-facing summary of changes
**Purpose:** Communication, not design

Pairs with:

- RFC → ADR → CHANGELOG

---

## 4.6 ROADMAP

**What:** Directional planning
**Not a contract**

Good for:

- Communicating vision
- Managing expectations

---

## 4.7 Threat Model / Security Review

**What:** Identifies risks & mitigations
**When:** Before public release

Especially important if:

- Executing templates
- Running scripts
- Downloading remote content

---

# 5. How They Fit Together (Lifecycle)

This is the **ideal flow**:

```
Idea
 ↓
RFC (proposal)
 ↓
Discussion / refinement
 ↓
Implementation
 ↓
ADR (record decision)
 ↓
Tests + CI
 ↓
Release
 ↓
CHANGELOG
```

---

# 6. What YOU Should Use for Scarff (Recommended)

Minimum but solid set:

```
docs/
 ├── rfcs/
 │   ├── 0001-new-command.md
 │   ├── 0002-template-system.md
 │
 ├── adrs/
 │   ├── 0001-rust-as-core-language.md
 │   ├── 0002-cocogitto-versioning.md
 │
 ├── roadmap.md
 ├── contributing.md
 └── security.md
```

---

# 7. Rule of Thumb (Memorable)

- **RFC** → _What should we build?_
- **ADR** → _Why did we build it this way?_
- **Tests** → _Does it actually work?_
- **CI** → _Can we trust every change?_

---

If you want next, I can:

- Convert an RFC into ADRs automatically
- Give you templates for RFC / ADR tailored to Scarff
- Help decide _what deserves an RFC vs ADR_

Just tell me.

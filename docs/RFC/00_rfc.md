# RFC-0001: `scarff new` Command Specification

**Status:** Draft
**Author:** oesisu
**Target Version:** v0.1.x (MVP)
**Last Updated:** 2026-02-17

---

## 1. Abstract

This RFC defines the behavior, guarantees, and evolution path of the `scarff new` command.
The command scaffolds fully runnable software projects with deterministic structure, strong validation, and sensible defaults.

The RFC serves as:

- A **design contract**
- A **reference for contributors**
- A **baseline for automated testing**

---

## 2. Motivation

Project scaffolding is often:

- Inconsistent
- Overly interactive
- Hard to reproduce
- Fragile across ecosystems

Scarff aims to solve this by making project generation:

- **Explicit**
- **Composable**
- **Deterministic**
- **Non-interactive by default**

The `new` command is the entry point for this philosophy.

---

## 3. Goals

### Primary Goals

- Generate **fully runnable projects**
- Support **multiple languages and frameworks**
- Provide **strong validation and inference**
- Enforce **predictable output**

### Non-Goals (for MVP)

- Interactive wizard UX
- GUI or web-based scaffolding
- Remote template fetching (planned later)

---

## 4. Command Definition

### Syntax

```text
scarff new <PROJECT_PATH> [OPTIONS]
```

### Core Options

| Flag | Long          | Description          |
| ---- | ------------- | -------------------- |
| `-l` | `--lang`      | Programming language |
| `-t` | `--type`      | Project kind         |
| `-a` | `--arch`      | Architecture         |
| `-f` | `--framework` | Framework            |

### Global Options

| Flag              | Description                    |
| ----------------- | ------------------------------ |
| `-h`, `--help`    | Context-aware help             |
| `-v`, `--verbose` | Increase verbosity (stackable) |
| `-q`, `--quiet`   | Errors only                    |
| `-y`, `--yes`     | Skip confirmation              |
| `--force`         | Overwrite existing files       |
| `--dry-run`       | Simulate without writing files |

---

## 5. Inference Rules

When flags are omitted:

1. **Language inference**
   - Required unless `--default` is used

2. **Framework inference**
   - Derived from `(language, project type)`

3. **Architecture inference**
   - Derived from `(language, project type)`

4. **Defaults**
   - Must be deterministic
   - Must be displayed to the user

> Explicit flags always override inference.

---

## 6. Invariants (Hard Guarantees)

These rules **must never be broken**.

1. **Determinism**
   - Same inputs → same structure, always

2. **Fail-fast validation**
   - Invalid combinations error before filesystem writes

3. **No partial state**
   - On failure, no files or directories remain

4. **Runnable by default**
   - Generated project must compile or run immediately

5. **Transparent inference**
   - All inferred values must be printed

6. **Explicit override precedence**
   - User flags override all defaults

---

## 7. Error Handling

Errors must be:

- Human-readable
- Actionable
- Contextual

Example:

```text
Error: Framework `react` is not compatible with project type `cli`
Hint: Try `--type frontend` or remove `--framework`
```

---

## 8. Output Contract

### Success Output

- Project path
- Language
- Project type
- Framework
- Architecture
- Next steps

### Dry Run Output

- Identical to real run
- Explicit notice:

  > “Dry run – no files were written”

---

## 9. Backward Compatibility

- Breaking changes require a new RFC
- Default behavior must not silently change
- New flags must not alter existing semantics

---

## 10. Future Extensions

- Interactive mode (`-i`)
- Remote template registry
- Feature flags (`--features`)
- User config (`~/.config/scarff/config.toml`)
- Plugin system

---

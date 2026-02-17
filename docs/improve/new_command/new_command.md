# Improvements & Design Goals for the `new` Command

This document outlines planned fixes, enhancements, behavioral guarantees, and future directions for the `scarff new` command.
The goal is to ensure the command is **predictable, ergonomic, extensible**, and aligned with Scarff’s long-term vision.

---

## Overview

The `new` command allows users to scaffold **fully runnable projects** from the command line with minimal friction.
It emphasizes:

- Sensible defaults
- Explicit overrides
- Strong validation
- Deterministic output
- Zero ambiguity in project structure

At this stage, the command is **non-interactive by default**.

---

## Purpose

The `new` command enables users to:

- Create projects quickly without boilerplate fatigue
- Rely on inferred defaults when options are omitted
- Explicitly control language, project type, architecture, and framework
- Produce a project that **builds and runs immediately**

---

## Holistic Usage Model

The command is designed to scale from **minimal input** to **fully explicit configuration**.

> ⚠️ At this phase, all configuration is provided via flags
> Interactivity may be introduced in the future

### Core Flags

```text
-l, --lang         Programming language
-t, --type         Project type (cli, backend, frontend, fullstack, worker, library)
-a, --arch         Architecture (layered, flat, hexagonal, etc.)
-f, --framework    Framework (axum, actix, fastapi, nextjs, etc.)
```

### Advanced / Future Flags

```text
-i                Interactive mode (planned)
--features         Optional feature set (future extension)
--template-path    Custom local template path
--template-id      Template identifier (local or remote)
--default          Use fully inferred defaults
```

---

## Usage Examples (From Minimal to Explicit)

### Minimal (Inference-driven)

```bash
scarff new my-api -l rust -t cli
```

### Explicit Configuration

```bash
scarff new my-api \
  --lang=rust \
  --type=backend \
  --framework=axum \
  --arch=layered
```

### Custom Project Location

```bash
scarff new ./my-api -l rust -t backend -f axum -a layered
```

### Custom Template Source

```bash
scarff new ./my-api \
  -l rust \
  -t backend \
  --framework=axum \
  --arch=layered \
  --template-path=/path/to/custom/templates
```

### Template ID

```bash
scarff new ./my-api --template-id=axum-layered-backend
```

### Fully Defaulted

```bash
scarff new ./my-api --default
```

---

## Global Flags

### Help

```text
-h, --help
```

- Available at **any command level**
- Explains valid combinations and inferred defaults

### Verbosity

```text
-v, --verbose   Increase log verbosity (stackable: -vvv)
-q, --quiet     Errors only
(default)       Info + warnings
```

### Confirmation & Safety

```text
-y, --yes       Skip confirmation prompts
--force         Overwrite existing files/directories
--dry-run       Simulate project generation without writing files
```

---

## Expected Output

When successful, the command should:

1. Clearly state what is being generated
2. Display inferred values (if any)
3. Indicate template source
4. Confirm filesystem changes
5. Provide next steps

Example:

```text
✔ Project created: my-api
✔ Language: Rust
✔ Type: Backend
✔ Framework: Axum
✔ Architecture: Layered

Next steps:
  cd my-api
  cargo run
```

In `--dry-run` mode:

- No filesystem changes
- Output mirrors real execution
- Explicitly states **“Dry run – no files were written”**

---

## Invariants (Non-Negotiable Guarantees)

These rules must **always hold**, regardless of future changes:

1. **Deterministic output**
   - Same inputs → same structure

2. **No silent assumptions**
   - Any inferred value must be shown to the user

3. **Fail fast**
   - Invalid combinations error early with actionable messages

4. **Runnable by default**
   - Generated projects must compile/run without extra steps

5. **No partial state**
   - On failure, no half-generated projects remain

6. **Explicit overrides win**
   - User-provided flags always override inference

---

## Future Improvements

### Planned Enhancements

- Interactive mode (`-i`)
- Remote template registry
- Feature flags (`--features`)
- Config-driven defaults (`~/.config/scarff/config.toml`)
- Plugin system for custom generators

### Known Areas for Refinement

- Better framework ↔ project type inference
- Smarter defaults per language ecosystem
- More expressive error diagnostics
- Improved UX for multi-package / monorepo generation

---

## Summary

The `new` command is the **foundation of Scarff’s user experience**.
Every improvement should reinforce:

- Clarity over cleverness
- Explicitness over magic
- Stability over convenience

This document serves as the **design contract** for its behavior and evolution.

---

If you want, next we can:

- Turn this into a **formal RFC**
- Map it directly to your Rust code structure
- Define a **test matrix** that enforces these invariants

Just tell me where you want to go next.

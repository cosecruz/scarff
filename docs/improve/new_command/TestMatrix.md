# Test Matrix: `scarff new`

This matrix **enforces the RFC invariants**.

---

## 1. Argument Validation Tests

| Test Case          | Input                | Expected Result |
| ------------------ | -------------------- | --------------- |
| Missing lang       | `scarff new app`     | Error           |
| Invalid lang       | `--lang java`        | Error           |
| Invalid type       | `--type foo`         | Error           |
| Invalid framework  | `--framework rails`  | Error           |
| Incompatible combo | `rust + react + cli` | Error           |

---

## 2. Inference Tests

| Input               | Expected Inference    |
| ------------------- | --------------------- |
| `-l rust -t cli`    | Default Rust CLI arch |
| `-l ts -t frontend` | React/Vite (default)  |
| `--default`         | Fully inferred stack  |

Assertions:

- Inferred values appear in output
- Same inputs → same result

---

## 3. Filesystem Safety Tests

| Scenario                       | Assertion         |
| ------------------------------ | ----------------- |
| Failure mid-run                | No files created  |
| `--dry-run`                    | No files created  |
| Existing dir without `--force` | Error             |
| With `--force`                 | Overwrites safely |

---

## 4. Output Tests

| Flag    | Expected Behavior |
| ------- | ----------------- |
| Default | Info + warnings   |
| `-v`    | Debug output      |
| `-vvv`  | Trace-level       |
| `-q`    | Errors only       |

---

## 5. Runnable Guarantee Tests

| Language    | Command       | Assertion |
| ----------- | ------------- | --------- |
| Rust        | `cargo build` | Success   |
| Rust CLI    | `cargo run`   | Runs      |
| TS frontend | `npm run dev` | Starts    |

---

## 6. Determinism Tests

| Test                     | Assertion        |
| ------------------------ | ---------------- |
| Same args twice          | Identical output |
| Different order of flags | Same result      |
| CI vs local              | Same structure   |

---

## 7. Regression Tests

Every bug fix adds:

- A failing test
- A passing test after fix

---

## 8. Integration Test Structure (Rust)

```text
tests/
 ├── new_basic.rs
 ├── new_inference.rs
 ├── new_invalid.rs
 ├── new_dry_run.rs
 ├── new_force.rs
 └── new_output.rs
```

---

## 9. CI Enforcement

Required in CI:

- Unit tests
- Integration tests
- Snapshot tests (optional)
- Deny + clippy + fmt

---

## Final Notes

This RFC + test matrix together form a **contract**:

- If behavior changes → update RFC
- If RFC changes → update tests

This keeps Scarff **disciplined, scalable, and production-grade**.

---

If you want next, I can:

- Map each RFC section to **actual Rust modules**
- Write **starter integration test files**
- Design a **template metadata spec** that makes inference trivial

Just say the word.

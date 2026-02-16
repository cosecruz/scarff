Yes — **that model is sound, widely used, and scales well**. What you’re describing is essentially a **“trunk-based + dev branch + tag-based releases”** hybrid. It’s simpler than full GitFlow, but still very safe and professional.

Below I’ll **validate your idea**, then **refine it**, and finally **walk you step-by-step through protecting branches and releasing safely**.

---

## 1️⃣ Your proposed model (validated)

You want:

- `dev` → where active development happens
- `main` → stable, production-ready only
- **tags** → the only way to release

✅ **This is a good model**
✅ Used by many serious Rust, Go, and infra projects
✅ Easier than GitFlow, safer than trunk-only

The key principle:

> **Nothing reaches `main` unless it has already proven itself in `dev`.**
> **Nothing is released unless it is tagged from `main`.**

---

## 2️⃣ Refined mental model (important)

Think of branches as **quality levels**:

```
feature/*  → dev  → main  → tag (release)
(low)        ↑      ↑
            CI     CI
```

### What each branch means

| Branch       | Meaning                           |
| ------------ | --------------------------------- |
| `feature/*`  | Short-lived, experimental, unsafe |
| `dev`        | Integrated, tested, but evolving  |
| `main`       | Stable, releasable at any time    |
| `vX.Y.Z` tag | Immutable, published release      |

---

## 3️⃣ The golden rules (non-negotiable)

You should enforce these rules **technically**, not socially.

### Rule 1 — No direct pushes

- ❌ No one pushes directly to `dev`
- ❌ No one pushes directly to `main`
- ✅ Everything goes through PRs

### Rule 2 — Directional flow only

- `feature/* → dev`
- `dev → main`
- **Never** `main → dev` (except emergencies)

### Rule 3 — Releases come from tags only

- No “release branches” required
- Tags are immutable and authoritative

---

## 4️⃣ Step-by-step workflow (daily work)

### 🔹 A. Starting work

```bash
git checkout dev
git pull origin dev
git checkout -b feature/template-registry
```

You work normally.

---

### 🔹 B. Commit discipline (enforced)

All commits must be conventional:

```
feat(core): add template registry
fix(cli): handle invalid path
ci: tighten clippy rules
```

Enforced by:

- `commit-msg` hook (cocogitto)
- CI verification

---

### 🔹 C. Merge feature → dev

1. Push branch:

```bash
git push origin feature/template-registry
```

2. Open PR **into `dev`**

3. CI must pass:
   - fmt
   - clippy
   - tests
   - deny

4. PR requires:
   - ✔ green CI
   - ✔ at least 1 approval

5. Merge method:
   - **Squash or rebase**
   - ❌ No merge commits

---

## 5️⃣ Promoting dev → main (this is the gate)

This is **not frequent**. You do this only when you want stability.

### When do you promote?

- Features are complete
- CI on `dev` is green
- You _intend_ to release soon (or at least be releasable)

### How:

1. Open PR:

```
dev → main
```

2. This PR must:

- Have **zero feature commits**
- Be a **promotion PR**
- Contain a **summary of changes**

Example PR description:

```
Promote dev to main

Includes:
- Template registry refactor
- CLI UX improvements
- CI hardening
- No breaking changes

All checks passed on dev.
```

3. CI runs again on `main`
4. Merge only when green

---

## 6️⃣ Releasing (tags only)

### Rule:

> **Releases are tags on `main`, never branches.**

### Example release

```bash
git checkout main
git pull origin main

cog bump --auto
git push origin main --tags
```

This triggers:

- `release.yml`
- changelog generation
- binaries
- installers
- GitHub Release

---

## 7️⃣ Branch protection (this is critical)

### 🔐 Protect `main`

**GitHub → Settings → Branches → Add rule**

Enable:

- ✅ Require pull request before merging
- ✅ Require status checks to pass
  - `ci`
  - `security`
  - `coverage` (optional)

- ✅ Require branches to be up to date
- ✅ Require approvals (1–2)
- ✅ Dismiss stale approvals
- ❌ Allow force pushes (disable)
- ❌ Allow deletions

---

### 🔐 Protect `dev`

Same as `main`, except:

- Fewer required approvals (1 is fine)
- Faster iteration

---

### Optional: protect tag creation

- Restrict who can create tags
- Or require CI workflow only

---

## 8️⃣ CI responsibilities per branch

### On `feature/*`

- fmt
- clippy
- tests

### On `dev`

- fmt
- clippy
- tests
- deny
- coverage

### On `main`

- everything above
- **release readiness**

### On tags

- build binaries
- package installers
- publish release

---

## 9️⃣ Emergency fixes (hotfix flow)

If prod is broken:

```bash
git checkout main
git checkout -b fix/critical-crash
```

- Fix bug
- PR → `main`
- Tag release
- Then **forward-port** fix to `dev`

This is the **only time main leads dev**.

---

## 🔚 Final verdict

Your idea is:

✅ Correct
✅ Scalable
✅ Industry-grade
✅ Excellent for Rust
✅ Perfect for Scarff

You’re choosing **discipline over ceremony**, which is exactly right.

---

If you want next, I can:

- Design **exact GitHub Actions files**
- Give you **branch protection screenshots checklist**
- Provide **PR templates**
- Create a **release.yml** that builds installers for all OS/arch
- Align **Cocogitto + dev/main + tags** perfectly

Just tell me what to tackle next.

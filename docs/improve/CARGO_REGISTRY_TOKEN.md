# 2️⃣ Configuring

Go to <https://crates.io>

Account → API Tokens

Create token:

Name: github-actions

Scope: publish

Copy it once

Step 2 — Add GitHub secret

Repo → Settings → Secrets and variables → Actions

Click New repository secret

Name:

CARGO_TOKEN

Value:

<paste crates.io token>

📌 Do not name it CARGO_REGISTRY_TOKEN here
GitHub secrets are injected manually.

Step 3 — Use it in workflow (correct way)
env:
CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_TOKEN }}

or (better, scoped):

- name: Publish crates
  run: cargo publish -p scarff-cli
  env:
  CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_TOKEN }}

📌 Cargo only recognizes CARGO_REGISTRY_TOKEN
📌 GitHub only stores secrets.\*

🔒 Security best practices (important)
✅ Do

Use repo-level secret, not org-level (unless shared)

Rotate token after release pipeline stabilizes

Scope token to publish only

❌ Don’t

Never echo it

Never put it in .cargo/config.toml

Never reuse personal tokens elsewhere

---

Next logical steps (tell me when ready)

🔁 Automate release tagging with cog

🔐 Protect main from non-dev PRs

🧪 Required-checks matrix tuning

📦 Dry-run publish validation

You’re setting this up the right way — this is exactly how mature Rust projects do it.

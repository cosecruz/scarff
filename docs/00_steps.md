# Integration steps

## 1. setup git

- to have pre gir and post git checking of commits, code checks, push standard- to have pre gir and post git checking of commits, code checks, push standard
- cannot push to main branch directly; have to do so via feature branches; short lived
- every commit and push must meet standard before allowed;
- push is merged via PR and must go

## 2. setup CI/CD

- github actions via .github/workflows
- ci.yaml and release.yaml
  -.github/workflows/ci.ymlQuality gate — runs on every push and PR.github/workflows/release.ymlRelease pipeline — triggered by version tags.github/dependabot.ymlKeeps Actions and Cargo deps updated automatically
- AND MORE

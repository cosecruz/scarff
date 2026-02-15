# Scarff Roadmap

&gt; A structured roadmap for the Scarff CLI scaffolding tool, organized by priority, magnitude, and implementation phases.

---

## Overview

This roadmap defines the evolution of Scarff from MVP to enterprise-grade developer tooling. Each phase builds upon the previous, with clear milestones and feature categorization.

**Legend:**

- 🔴 **Critical Path** - Must have for release
- 🟡 **High Priority** - Core differentiators
- 🟢 **Standard** - Expected features
- 🔵 **Future** - Innovation/experimental

---

## Phase 1: Foundation (MVP) 🔴

**Target:** Stable CLI with core scaffolding capabilities
**Magnitude:** Core Infrastructure
**Timeline:** Weeks 1-4

### 1.1 CLI Core & Project Generation

| Feature                | Description                                                                      | Tags                 |
| ---------------------- | -------------------------------------------------------------------------------- | -------------------- |
| **Project Creation**   | `scarff new &lt;name&gt;` creates project in current directory                   | `cli`, `core`        |
| **Path Resolution**    | Support absolute/relative paths: `scarff new /path/to/name` or `scarff new name` | `cli`, `filesystem`  |
| **Template Rendering** | Replace `{{PROJECT_NAME}}` and other placeholders during generation              | `templating`, `core` |
| **Output Cleanup**     | Clean, informative CLI output with progress indicators                           | `ux`, `cli`          |

- --force does not work
- output is a bit messy; divide output from logging how do production cli do it?
-

### 1.2 Template System

| Feature                | Description                                               | Tags                         |
| ---------------------- | --------------------------------------------------------- | ---------------------------- |
| **Template Discovery** | Load templates from `.config/scarff/templates/`           | `templates`, `config`        |
| **Built-in Templates** | Seed with Rust, TypeScript, Python templates on first run | `templates`, `bootstrap`     |
| **Template Metadata**  | `template.toml` with matchers, features, dependencies     | `templates`, `metadata`      |
| **User Templates**     | Add custom templates to config directory                  | `templates`, `extensibility` |

### 1.3 Configuration & Extensibility

| Feature                    | Description                                                  | Tags                      |
| -------------------------- | ------------------------------------------------------------ | ------------------------- |
| **Runtime Configuration**  | `.config/scarff/config.toml` for settings                    | `config`, `core`          |
| **Custom Capabilities**    | `.config/scarff/capabilities.toml` for user-defined matchers | `config`, `extensibility` |
| **Template Path Override** | CLI flag `--templates-path` for runtime specification        | `cli`, `config`           |

### 1.4 Quality & Documentation

| Feature                | Description                                           | Tags                |
| ---------------------- | ----------------------------------------------------- | ------------------- |
| **Code Documentation** | Comprehensive Rust docs and inline comments           | `docs`, `quality`   |
| **User Documentation** | scarff.oesisu.com/docs with guides and API reference  | `docs`, `web`       |
| **Landing Page**       | scarff.oesisu.com with /about, /download, /contribute | `web`, `marketing`  |
| **Release Pipeline**   | CI/CD for automated builds, tests, releases           | `devops`, `release` |

### Phase 1 Exit Criteria

- [ ] `scarff new my-api` creates `./my-api/` correctly
- [ ] `scarff new /abs/path/my-api` creates at absolute path
- [ ] Templates load from `.config/scarff/templates/`
- [ ] Placeholders render correctly
- [ ] All templates generate valid, runnable projects
- [ ] Documentation site live
- [ ] Released to crates.io / package managers

---

## Phase 2: Enhanced Developer Experience 🟡

**Target:** Professional-grade tooling with rich features
**Magnitude:** Feature Expansion
**Timeline:** Weeks 5-10

### 2.1 CLI Interactivity

| Feature                | Description                                             | Tags                        |
| ---------------------- | ------------------------------------------------------- | --------------------------- |
| **Interactive Mode**   | `scarff new` with TUI prompts for options               | `cli`, `tui`, `ux`          |
| **Template Selection** | Fuzzy finder for template selection                     | `cli`, `tui`, `ux`          |
| **Feature Flags**      | Interactive checkbox for optional features              | `cli`, `tui`, `scaffolding` |
| **Validation**         | Pre-flight checks for dependencies (Node, Python, etc.) | `cli`, `validation`         |

### 2.2 Scaffolding Features (Feature Tags)

| Feature                    | Description                          | Tags                                | Phase 1 Templates   |
| -------------------------- | ------------------------------------ | ----------------------------------- | ------------------- |
| **Caching**                | Redis, Memcached integration         | `cache`, `redis`, `performance`     | ✅ Rust, TS, Python |
| **Authentication**         | JWT, OAuth2, Session auth            | `auth`, `jwt`, `security`           | ✅ Rust, TS, Python |
| **Docker**                 | Dockerfile + compose generation      | `docker`, `containerization`        | ✅ All              |
| **Database**               | SQL/NoSQL ORM setup                  | `database`, `orm`, `sql`            | ✅ All              |
| **CI/CD**                  | GitHub Actions, GitLab CI templates  | `ci-cd`, `devops`, `github-actions` | ⬜                  |
| **Scripting**              | Bash, Python utility scripts         | `scripting`, `automation`           | ⬜                  |
| **Testing**                | Unit, integration, e2e test setup    | `testing`, `quality`                | ✅ All              |
| **API Documentation**      | OpenAPI/Swagger auto-generation      | `docs`, `api`                       | ✅ All              |
| **Monitoring**             | Health checks, metrics endpoints     | `monitoring`, `observability`       | ⬜                  |
| **Logging**                | Structured logging setup             | `logging`, `observability`          | ✅ All              |
| **Environment Management** | .env, config files, secrets          | `config`, `security`                | ✅ All              |
| **WebSockets**             | Real-time communication setup        | `websocket`, `real-time`            | ⬜                  |
| **GraphQL**                | Schema and resolver scaffolding      | `graphql`, `api`                    | ⬜                  |
| **gRPC**                   | Protocol buffer service setup        | `grpc`, `microservices`             | ⬜                  |
| **Message Queue**          | RabbitMQ, Kafka, SQS integration     | `messaging`, `async`                | ⬜                  |
| **Search**                 | Elasticsearch, Typesense integration | `search`, `elasticsearch`           | ⬜                  |
| **Storage**                | S3, MinIO, local file handling       | `storage`, `files`                  | ⬜                  |
| **Email**                  | SMTP, SendGrid, Mailgun setup        | `email`, `notifications`            | ⬜                  |
| **Payments**               | Stripe, PayPal integration scaffold  | `payments`, `ecommerce`             | ⬜                  |
| **Background Jobs**        | Bull, Celery, Sidekiq setup          | `jobs`, `queue`, `async`            | ⬜                  |

### 2.3 Template Registry (Scarff Temple)

| Feature             | Description                                     | Tags                      |
| ------------------- | ----------------------------------------------- | ------------------------- |
| **Registry CRUD**   | Create, read, update, delete templates remotely | `registry`, `cloud`       |
| **Version Control** | Template versioning with semver                 | `registry`, `versioning`  |
| **Public/Private**  | Visibility controls for templates               | `registry`, `security`    |
| **CLI Integration** | `scarff push`, `scarff pull` for templates      | `cli`, `registry`         |
| **Caching**         | Local cache of registry templates               | `registry`, `performance` |
| **Indexing**        | Search and filter registry templates            | `registry`, `search`      |

### 2.4 Language & Framework Expansion

| Language/Framework   | Architecture       | Priority | Tags                             |
| -------------------- | ------------------ | -------- | -------------------------------- |
| Go (Gin/Fiber)       | Clean Architecture | High     | `go`, `web`, `microservices`     |
| Java (Spring Boot)   | Modular Monolith   | High     | `java`, `enterprise`, `spring`   |
| C# (.NET)            | Clean Architecture | High     | `csharp`, `dotnet`, `enterprise` |
| PHP (Laravel)        | MVC                | Medium   | `php`, `laravel`, `mvc`          |
| Ruby (Rails)         | MVC                | Medium   | `ruby`, `rails`, `mvc`           |
| Elixir (Phoenix)     | Layered            | Medium   | `elixir`, `phoenix`, `real-time` |
| Kotlin (Ktor/Spring) | Clean Architecture | Medium   | `kotlin`, `jvm`, `android`       |
| Swift (Vapor)        | MVC                | Low      | `swift`, `ios`, `backend`        |
| Dart (Serverpod)     | MVC                | Low      | `dart`, `flutter`, `fullstack`   |

### Phase 2 Exit Criteria

- [ ] Interactive TUI for project creation
- [ ] 10+ language/framework templates
- [ ] Scarff Temple registry operational
- [ ] 20+ feature tags supported
- [ ] Template versioning working

---

## Phase 3: Enterprise & Ecosystem 🟢

**Target:** Industry-standard tooling with ecosystem integration
**Magnitude:** Platform Expansion
**Timeline:** Weeks 11-20

### 3.1 DevOps & Infrastructure

| Feature               | Description                           | Tags                                 |
| --------------------- | ------------------------------------- | ------------------------------------ |
| **Kubernetes**        | K8s manifests, Helm charts generation | `kubernetes`, `k8s`, `orchestration` |
| **Terraform**         | IaC scaffolding for AWS, Azure, GCP   | `terraform`, `iac`, `cloud`          |
| **AWS Integration**   | SAM, CDK, CloudFormation templates    | `aws`, `cloud`, `serverless`         |
| **Azure Integration** | Bicep, ARM templates                  | `azure`, `cloud`, `microsoft`        |
| **GCP Integration**   | Cloud Deployment Manager              | `gcp`, `google-cloud`, `cloud`       |
| **GitOps**            | ArgoCD, Flux configuration            | `gitops`, `cd`, `kubernetes`         |
| **Monitoring Stack**  | Prometheus, Grafana, Loki setup       | `monitoring`, `observability`        |
| **Service Mesh**      | Istio, Linkerd configuration          | `servicemesh`, `microservices`       |

### 3.2 Development Tooling

| Feature              | Description                       | Tags                                 |
| -------------------- | --------------------------------- | ------------------------------------ |
| **LSP Integration**  | Language server configuration     | `lsp`, `ide`, `developer-experience` |
| **Pre-commit Hooks** | Husky, lint-staged setup          | `git`, `quality`, `automation`       |
| **IDE Configs**      | VSCode, IntelliJ, Neovim settings | `ide`, `developer-experience`        |
| **Debugging**        | Launch configs, debug scripts     | `debugging`, `developer-experience`  |
| **Hot Reload**       | File watching, auto-restart       | `dx`, `developer-experience`         |

### 3.3 Project Lifecycle Management

| Feature                  | Description                            | Tags                               |
| ------------------------ | -------------------------------------- | ---------------------------------- |
| **`.scarff/` Directory** | Per-project Scarff metadata and config | `project-management`, `metadata`   |
| **Project Tracking**     | IDE integration, progress tracking     | `project-management`, `analytics`  |
| **Rules Engine**         | Custom scaffolding rules per project   | `extensibility`, `rules`           |
| **Plugins (WASM)**       | WebAssembly plugin system              | `plugins`, `extensibility`, `wasm` |
| **Git Integration**      | Native Git operations, changelog gen   | `git`, `vcs`, `automation`         |
| **Multi-VCS**            | Mercurial, SVN, Perforce support       | `vcs`, `enterprise`                |

### 3.4 Workspace Management

| Feature                   | Description                         | Tags                                |
| ------------------------- | ----------------------------------- | ----------------------------------- |
| **Monorepo Support**      | Turborepo, Nx, Rush integration     | `monorepo`, `workspace`             |
| **Workspace Templates**   | Multi-package project scaffolding   | `workspace`, `enterprise`           |
| **Dependency Management** | Cross-package dependency resolution | `workspace`, `dependencies`         |
| **Unified Tooling**       | Shared configs across workspace     | `workspace`, `developer-experience` |

### Phase 3 Exit Criteria

- [ ] Kubernetes and Terraform generation
- [ ] Cloud provider integrations (AWS/Azure/GCP)
- [ ] WASM plugin system operational
- [ ] `.scarff/` directory with project tracking
- [ ] Monorepo workspace support

---

## Phase 4: Intelligence & Innovation 🔵

**Target:** AI-powered, next-generation developer tooling
**Magnitude:** Research & Development
**Timeline:** Months 5-12

### 4.1 AI Integration

| Feature                     | Description                            | Tags                                |
| --------------------------- | -------------------------------------- | ----------------------------------- |
| **Prompt-to-Template**      | Natural language to scaffold selection | `ai`, `nlp`, `search`               |
| **Vector Search**           | Semantic template matching             | `ai`, `vector-db`, `search`         |
| **Smart Defaults**          | ML-based feature recommendations       | `ai`, `ml`, `personalization`       |
| **Code Generation**         | AI-assisted template customization     | `ai`, `code-generation`, `gpt`      |
| **Claude Code Integration** | TUI-based AI pair programming          | `ai`, `tui`, `developer-experience` |

### 4.2 Analytics & Observability

| Feature                  | Description                        | Tags                       |
| ------------------------ | ---------------------------------- | -------------------------- |
| **Usage Analytics**      | Opt-in telemetry for feature usage | `analytics`, `privacy`     |
| **Developer Habits**     | Language/framework trend analysis  | `analytics`, `insights`    |
| **Template Performance** | Success rates, generation times    | `analytics`, `performance` |
| **Community Metrics**    | Popular templates, features        | `analytics`, `community`   |

### 4.3 Enterprise Features

| Feature                | Description                 | Tags                                      |
| ---------------------- | --------------------------- | ----------------------------------------- |
| **SSO/SAML**           | Enterprise authentication   | `enterprise`, `security`, `sso`           |
| **Audit Logging**      | Compliance and governance   | `enterprise`, `security`, `compliance`    |
| **Private Registries** | On-premise template hosting | `enterprise`, `registry`, `self-hosted`   |
| **RBAC**               | Role-based template access  | `enterprise`, `security`, `authorization` |
| **SLA/Support**        | Enterprise support tiers    | `enterprise`, `business`                  |

### Phase 4 Exit Criteria

- [ ] AI-powered template selection
- [ ] Analytics dashboard operational
- [ ] Enterprise features available
- [ ] Revenue model established

---

## Feature Tags Taxonomy

### By Category

**Language:** `rust`, `typescript`, `python`, `go`, `java`, `csharp`, `php`, `ruby`, `elixir`, `kotlin`, `swift`, `dart`

**Framework:** `nestjs`, `fastapi`, `axum`, `express`, `django`, `laravel`, `rails`, `spring`, `dotnet`

**Architecture:** `clean-architecture`, `hexagonal`, `modular-monolith`, `microservices`, `mvc`, `layered`, `cqrs`, `event-driven`

**Infrastructure:** `docker`, `kubernetes`, `terraform`, `aws`, `azure`, `gcp`, `ci-cd`, `github-actions`

**Data:** `database`, `orm`, `sql`, `nosql`, `redis`, `cache`, `search`, `elasticsearch`

**Communication:** `api`, `rest`, `graphql`, `grpc`, `websocket`, `messaging`, `kafka`, `rabbitmq`

**Security:** `auth`, `jwt`, `oauth`, `encryption`, `security`, `rbac`

**Operations:** `monitoring`, `logging`, `observability`, `tracing`, `metrics`

**Developer Experience:** `testing`, `docs`, `lsp`, `ide`, `debugging`, `hot-reload`

**Business:** `payments`, `email`, `notifications`, `storage`, `jobs`, `queue`

---

## Implementation Priority Matrix

| Priority | Phase | Features                       | Business Value | Technical Complexity |
| -------- | ----- | ------------------------------ | -------------- | -------------------- |
| P0       | 1     | Core CLI, templates, rendering | Critical       | Low-Medium           |
| P1       | 1     | Config system, docs, release   | Critical       | Low                  |
| P2       | 2     | Interactivity, feature tags    | High           | Medium               |
| P3       | 2     | Registry, more languages       | High           | Medium               |
| P4       | 3     | DevOps, K8s, cloud             | High           | High                 |
| P5       | 3     | Workspace, plugins             | Medium         | High                 |
| P6       | 4     | AI, analytics                  | High           | Very High            |
| P7       | 4     | Enterprise                     | Medium         | Medium               |

---

## Success Metrics

| Metric               | Target   | Phase |
| -------------------- | -------- | ----- |
| Downloads/Installs   | 1,000+   | 1     |
| Active Users         | 500+     | 2     |
| Community Templates  | 50+      | 2     |
| GitHub Stars         | 500+     | 2     |
| Enterprise Customers | 5+       | 3     |
| Revenue              | $10k MRR | 4     |

---

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for development setup and guidelines.

## License

MIT License - see [LICENSE](./LICENSE) for details.

> Can even integrate quiet ads that wll advertise tools based on user activity/ personalization or problem they want

VERB Persona
VERB Challenges
VERB Card
VERB To'and'Fro
VERB RoR (Rest'or'Rant)
Ahia - MarketPlace
Ann/Ada - Fashion
Dada/Cath - Homes/Hotels/Reserves
Focus
Kachy/Kachi - Entertainment/Events/Fun/Sports

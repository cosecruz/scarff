# Scarff

**Project scaffolding made instant.**

Scarff is a CLI tool that generates production-ready project structures in seconds. Choose your language, framework, and architecture—Scarff handles the boilerplate so you can focus on building.

```bash
scarff new my-app --lang rust --framework axum --arch hexagonal --type web-backend
cd my-app
cargo build && cargo run
# Server running on http://localhost:3000
```

---

## Features

- ⚡ **Instant Setup**: Go from idea to coding in under 60 seconds
- 🏗️ **Architecture-Aware**: Generates proper layered or hexagonal structures
- 🔧 **Framework-Ready**: Pre-configured with idiomatic framework conventions
- 🚀 **Zero Config**: Generated projects build and run immediately
- 📦 **Standalone**: No runtime dependencies—Scarff disappears after scaffolding
- 🎯 **Deterministic**: Same inputs always produce identical outputs

---

## Installation

### Download Binary (Recommended)

```bash
# macOS / Linux
#curl -fsSL https://scarff.dev/install.sh | sh

# Or download from releases
# https://github.com/yourusername/scarff/releases
```

### Build from Source

```bash
# git clone https://github.com/yourusername/scarff.git
# cd scarff
# cargo build --release
# sudo mv target/release/scarff /usr/local/bin/
```

### Verify Installation

```bash
scarff --version
# scarff 0.1.0
```

---

## Quick Start

### Create a Rust CLI Application

```bash
scarff new my-cli --lang rust --type cli --arch layered
cd my-cli
cargo run
```

### Create a Python Web Backend

```bash
scarff new my-api --lang python --framework fastapi --type web-backend --arch layered
cd my-api
python -m venv venv
source venv/bin/activate
pip install -r requirements.txt
uvicorn main:app --reload
```

### Create a TypeScript React SPA

```bash
scarff new my-app --lang typescript --framework react --type spa --arch layered
cd my-app
npm install
npm run dev
```

---

## Usage

### Basic Command Structure

```bash
scarff new <project-name> [OPTIONS]
```

### Required Options

| Option   | Description          | Values                                 |
| -------- | -------------------- | -------------------------------------- |
| `--lang` | Programming language | `rust`, `python`, `typescript`         |
| `--type` | Application type     | `cli`, `web-backend`, `spa`, `library` |
| `--arch` | Architecture style   | `layered`, `hexagonal`                 |

### Optional Options

| Option        | Description      | Example                              |
| ------------- | ---------------- | ------------------------------------ |
| `--framework` | Framework choice | `axum`, `fastapi`, `nestjs`, `react` |
| `--output`    | Output directory | `--output ~/projects`                |

---

## Supported Stacks

### Rust

| Type        | Frameworks | Architectures      |
| ----------- | ---------- | ------------------ |
| CLI         | (none)     | Layered            |
| Web Backend | Axum       | Layered, Hexagonal |
| Library     | (none)     | Layered            |

### Python

| Type        | Frameworks | Architectures      |
| ----------- | ---------- | ------------------ |
| CLI         | (none)     | Simple, Layered    |
| Web Backend | FastAPI    | Layered, Hexagonal |
| Library     | (none)     | Simple             |

### TypeScript

| Type        | Frameworks | Architectures      |
| ----------- | ---------- | ------------------ |
| CLI         | (none)     | Simple, Layered    |
| Web Backend | NestJS     | Layered, Hexagonal |
| SPA         | React      | Layered            |
| Library     | (none)     | Simple             |

---

## Examples

### Full Example: Rust Hexagonal Web Service

```bash
# Create the project
scarff new payment-service \
  --lang rust \
  --framework axum \
  --type web-backend \
  --arch hexagonal

# Navigate and build
cd payment-service
cargo build

# Generated structure:
# payment-service/
# ├── src/
# │   ├── domain/          # Business logic
# │   ├── application/     # Use cases
# │   ├── infrastructure/  # External integrations
# │   └── main.rs
# ├── Cargo.toml
# ├── .gitignore
# └── README.md

cargo run
# Server running on http://localhost:3000
```

### TypeScript + NestJS API

```bash
scarff new user-service \
  --lang typescript \
  --framework nestjs \
  --type web-backend \
  --arch hexagonal

cd user-service
npm install
npm run start:dev

# Generated structure follows hexagonal architecture:
# user-service/
# ├── src/
# │   ├── domain/
# │   ├── application/
# │   ├── infrastructure/
# │   └── main.ts
# ├── package.json
# ├── tsconfig.json
# └── nest-cli.json
```

---

## What Gets Generated

Every scaffolded project includes:

✅ **Complete directory structure** matching chosen architecture
✅ **Build configuration** (Cargo.toml, package.json, etc.)
✅ **Dependency declarations** for chosen framework
✅ **Entry point** with minimal working code
✅ **README** with project-specific instructions
✅ **.gitignore** with language-appropriate exclusions

### What Does NOT Get Generated

❌ Business logic (that's your job!)
❌ Database schemas or migrations
❌ CI/CD configuration
❌ Deployment scripts
❌ Test files (beyond framework defaults)

Scarff scaffolds structure, not implementation.

---

## Architecture Styles

### Layered Architecture

Clean separation of concerns across horizontal layers:

```
├── presentation/   # UI, API endpoints
├── business/       # Core logic
└── data/          # Database, external services
```

**Best for**: Traditional web apps, APIs, simple services

### Hexagonal Architecture (Ports & Adapters)

Domain-centric design with explicit boundaries:

```
├── domain/         # Pure business logic (no dependencies)
├── application/    # Use cases, orchestration
└── infrastructure/ # External integrations (DB, HTTP, etc.)
```

**Best for**: Complex domains, microservices, DDD applications

---

## Philosophy

Scarff follows these principles:

1. **Scaffolding, Not Management**: Scarff creates projects, then disappears. No lock-in.
2. **Idempotent by Design**: Same inputs = same outputs. Always.
3. **Convention Over Configuration**: Follows language/framework community standards.
4. **Zero Runtime Dependency**: Generated projects never require Scarff to build or run.
5. **Stateless**: Each invocation is independent. No hidden config files.

---

## Documentation

- [Full Documentation](./docs/README.md)
- [Architecture Overview](./docs/03_Design/01_Architecture.md)
- [Supported Templates](./docs/templates.md)
- [Design Decisions (ADRs)](./docs/03_Design/ADR/)

---

## Contributing

Contributions welcome! Please read our [Contributing Guide](CONTRIBUTING.md).

### Quick Start for Contributors

```bash
# Fork and clone
git clone https://github.com/yourusername/scarff.git
cd scarff

# Build
cargo build

# Run tests
cargo test

# Run locally
cargo run -- new test-project --lang rust --type cli --arch layered

# Verify generated project works
cd test-project && cargo build && cargo run
```

See [Development Guide](./docs/DEVELOPMENT.md) for details.

---

## Roadmap

### MVP (v0.1.0) - Current

- [x] Core scaffolding engine
- [x] Rust, Python, TypeScript support
- [x] Layered and Hexagonal architectures
- [x] CLI, Web Backend, SPA project types
- [x] Selected frameworks (Axum, FastAPI, NestJS, React)

### Post-MVP

- [ ] Custom user templates
- [ ] Interactive mode (wizard)
- [ ] More languages (Go, Java, C#)
- [ ] More architectures (Clean, DDD tactical patterns)
- [ ] Monorepo scaffolding
- [ ] Template validation tool

See [Issues](https://github.com/yourusername/scarff/issues) for full roadmap.

---

## FAQ

### Why use Scarff instead of create-react-app, cargo new, etc.?

Language-specific tools generate basic structures. Scarff generates _architecture-aligned_ structures with framework integration. You get hexagonal or layered from the start, not a flat directory.

### Does Scarff work offline?

Yes. All templates are embedded. No network calls.

### Can I customize templates?

Not in v0.1 (MVP). Custom templates are planned for post-MVP.

### What if I want a different architecture later?

Scarff only scaffolds. After that, the project is yours to refactor freely. Scarff won't interfere.

### Does this replace [tool X]?

Scarff complements language-specific tools. It's like `create-react-app` or `cargo new`, but architecture-aware and cross-language.

### Is this production-ready?

The MVP is suitable for new projects and experimentation. Review generated code before production use.

---

## Comparisons

| Tool             | Scope                      | Languages | Architecture | Frameworks |
| ---------------- | -------------------------- | --------- | ------------ | ---------- |
| **Scarff**       | Cross-language scaffolding | 3+        | Enforced     | Multiple   |
| cargo new        | Rust only                  | 1         | None         | None       |
| create-react-app | React only                 | 1         | None         | 1          |
| Yeoman           | Generator framework        | Any       | Varies       | Varies     |
| cookiecutter     | Template engine            | Any       | None         | None       |

---

## Support

- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/yourusername/scarff/issues)
- 💡 **Feature Requests**: [GitHub Discussions](https://github.com/yourusername/scarff/discussions)
- 📧 **Email**: <support@scarff.dev>
- 💬 **Discord**: [Join community](https://discord.gg/scarff) (coming soon)

---

## License

MIT License - see [LICENSE](LICENSE) file for details.

---

## Acknowledgments

- Inspired by the frustration of manual project setup
- Built with [clap](https://github.com/clap-rs/clap) for CLI parsing
- Thanks to the Rust, Python, and TypeScript communities for excellent tooling

---

**Made with ⚡ by [Your Name](https://github.com/yourusername)**

_Scarff: Because life's too short for boilerplate._

---

## Project Structure

```md
scarff/
├── crates/
│ ├── core/ # Pure business logic (no I/O, no CLI dependencies)
│ │ ├── src/
│ │ │ ├── lib.rs
│ │ │ ├── domain/ # Domain types (Target, Template, etc.)
│ │ │ ├── template/ # Template management (resolver, renderer, store)
│ │ │ └── scaffold/ # Scaffolding orchestration
│ │ └── Cargo.toml
│ └── cli/ # CLI interface (depends on core)
│ ├── src/
│ │ ├── main.rs
│ │ ├── args.rs # CLI argument parsing
│ │ ├── commands.rs # Command handlers
│ │ └── output.rs # User-facing messages and formatting
│ └── Cargo.toml
├── tests/ # Integration tests
├── examples/ # Usage examples
├── .github/ # CI/CD
├── Cargo.toml # Workspace root
├── rust-toolchain.toml # Rust version pinning
└── bacon.toml # Background checker config
```

---

**Version**: 0.1.0
**Last Updated**: 2026-02-03

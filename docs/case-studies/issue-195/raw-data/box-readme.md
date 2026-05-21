# box

A Docker container image that contains most popular language runtimes and tools to be reused for multiple software development tasks.

## Overview

This box provides a pre-configured development environment with common language runtimes installed. It is designed to be AI-agnostic and can be used as a base image for various development workflows.

## Included Runtimes & Tools

### Programming Languages
- **Node.js 20** (via NVM) + npm + Bun + Deno
- **Python** (latest stable via pyenv)
- **Go** (latest stable)
- **Rust** (via rustup) + Cargo
- **Java 21 LTS** (Eclipse Temurin via SDKMAN)
- **Kotlin** (via SDKMAN)
- **PHP 8.3** (via Homebrew)
- **Perl** (latest stable via Perlbrew)
- **Ruby** (latest stable via rbenv)
- **Swift 6.x** (latest stable)
- **R** (latest stable)
- **.NET SDK 8.0**
- **Assembly** (GNU Assembler, NASM, LLVM-MC, FASM)

### Theorem Provers
- **Lean** (via elan)
- **Rocq/Coq** (via opam)

### Build Tools
- **CMake**
- **Make**
- **GCC/G++**
- **Clang/Clang++**
- **LLVM**
- **LLD Linker**

### Development Tools
- **Git**
- **GitHub CLI (gh)**
- **Homebrew**

## Modular Architecture

The box is split into modular components, allowing you to use only what you need:

```
JS box (konard/box-js)
  └─ Essentials box (konard/box-essentials)
       ├─ box-python     (built in parallel)
       ├─ box-go         (built in parallel)
       ├─ box-rust       (built in parallel)
       ├─ box-java       (built in parallel)
       ├─ box-kotlin     (built in parallel)
       ├─ box-ruby       (built in parallel)
       ├─ box-php        (built in parallel)
       ├─ box-perl       (built in parallel)
       ├─ box-swift      (built in parallel)
       ├─ box-lean       (built in parallel)
       └─ box-rocq       (built in parallel)
            └─ Full box (konard/box) ← merges all via COPY --from

dind-box variants (issue #80, Docker-in-Docker):
  Any of the boxes above also has a "dind" sibling that adds a working
  Docker Engine on top of the source image:

    konard/box-js          → konard/box-js-dind
    konard/box-essentials  → konard/box-essentials-dind
    konard/box-<language>  → konard/box-<language>-dind
    konard/box (full)      → konard/box-dind
```

| Image | Description | Base Image |
|-------|-------------|------------|
| `konard/box` | Full box (all languages) | Assembled from all language images |
| `konard/box-essentials` | Essentials (git identity tools) | Built on JS box |
| `konard/box-js` | JavaScript only | Ubuntu 24.04 |
| `konard/box-python` | Python (pyenv) | Built on essentials |
| `konard/box-go` | Go (latest stable) | Built on essentials |
| `konard/box-rust` | Rust (rustup + cargo) | Built on essentials |
| `konard/box-java` | Java 21 (SDKMAN + Temurin) | Built on essentials |
| `konard/box-kotlin` | Kotlin (SDKMAN) | Built on essentials |
| `konard/box-ruby` | Ruby (rbenv) | Built on essentials |
| `konard/box-php` | PHP 8.3 (Homebrew) | Built on essentials |
| `konard/box-perl` | Perl (Perlbrew) | Built on essentials |
| `konard/box-swift` | Swift 6.x | Built on essentials |
| `konard/box-lean` | Lean (elan) | Built on essentials |
| `konard/box-rocq` | Rocq/Coq (Opam) | Built on essentials |
| `konard/box-dind` | Full box + Docker-in-Docker (issue #80) | Layered on Full box |
| `konard/box-<lang>-dind` | Any language box + Docker-in-Docker (issue #80) | Layered on each language box |

### Per-Language Install Scripts & Dockerfiles

Each language has its own standalone `install.sh` and `Dockerfile` under `ubuntu/24.04/<language>/`:

| Language | Directory | Key Tools |
|----------|-----------|-----------|
| JavaScript/TypeScript | `ubuntu/24.04/js/` | NVM, Node.js, Bun, Deno, npm |
| Python | `ubuntu/24.04/python/` | Pyenv, latest stable Python |
| Go | `ubuntu/24.04/go/` | Latest stable Go |
| Rust | `ubuntu/24.04/rust/` | rustup, Cargo |
| Java | `ubuntu/24.04/java/` | SDKMAN, Eclipse Temurin 21 |
| Kotlin | `ubuntu/24.04/kotlin/` | SDKMAN, Kotlin |
| .NET | `ubuntu/24.04/dotnet/` | .NET SDK 8.0 |
| R | `ubuntu/24.04/r/` | R base |
| Ruby | `ubuntu/24.04/ruby/` | rbenv, latest Ruby 3.x |
| PHP | `ubuntu/24.04/php/` | Homebrew, PHP 8.3 |
| Perl | `ubuntu/24.04/perl/` | Perlbrew, latest Perl |
| Swift | `ubuntu/24.04/swift/` | Swift 6.x |
| Lean | `ubuntu/24.04/lean/` | elan, Lean prover |
| Rocq/Coq | `ubuntu/24.04/rocq/` | Opam, Rocq prover |
| C/C++ | `ubuntu/24.04/cpp/` | CMake, Clang, LLVM, LLD |
| Assembly | `ubuntu/24.04/assembly/` | NASM, FASM (x86_64) |
| Docker-in-Docker | `ubuntu/24.04/dind/` | Docker CE, Buildx, Compose, dockerd entrypoint (issue #80) |

Each install script can be run standalone on Ubuntu 24.04:

```bash
# Install just Go on your Ubuntu 24.04 system
curl -fsSL https://raw.githubusercontent.com/link-foundation/box/main/ubuntu/24.04/go/install.sh | bash
```

## Docker Images

### Docker Hub - Combo Boxes

| Image | Multi-arch | AMD64 | ARM64 |
|-------|------------|-------|-------|
| Full Box | [`konard/box:latest`](https://hub.docker.com/r/konard/box/tags?name=latest) | [`latest-amd64`](https://hub.docker.com/r/konard/box/tags?name=latest-amd64) | [`latest-arm64`](https://hub.docker.com/r/konard/box/tags?name=latest-arm64) |
| Essentials | [`konard/box-essentials:latest`](https://hub.docker.com/r/konard/box-essentials/tags?name=latest) | [`latest-amd64`](https://hub.docker.com/r/konard/box-essentials/tags?name=latest-amd64) | [`latest-arm64`](https://hub.docker.com/r/konard/box-essentials/tags?name=latest-arm64) |
| JS | [`konard/box-js:latest`](https://hub.docker.com/r/konard/box-js/tags?name=latest) | [`latest-amd64`](https://hub.docker.com/r/konard/box-js/tags?name=latest-amd64) | [`latest-arm64`](https://hub.docker.com/r/konard/box-js/tags?name=latest-arm64) |

### Docker Hub - Language Boxes

| Language | Multi-arch | AMD64 | ARM64 |
|----------|------------|-------|-------|
| Python | [`konard/box-python:latest`](https://hub.docker.com/r/konard/box-python/tags?name=latest) | [`latest-amd64`](https://hub.docker.com/r/konard/box-python/tags?name=latest-amd64) | [`latest-arm64`](https://hub.docker.com/r/konard/box-python/tags?name=latest-arm64) |
| Go | [`konard/box-go:latest`](https://hub.docker.com/r/konard/box-go/tags?name=latest) | [`latest-amd64`](https://hub.docker.com/r/konard/box-go/tags?name=latest-amd64) | [`latest-arm64`](https://hub.docker.com/r/konard/box-go/tags?name=latest-arm64) |
| Rust | [`konard/box-rust:latest`](https://hub.docker.com/r/konard/box-rust/tags?name=latest) | [`latest-amd64`](https://hub.docker.com/r/konard/box-rust/tags?name=latest-amd64) | [`latest-arm64`](https://hub.docker.com/r/konard/box-rust/tags?name=latest-arm64) |
| Java | [`konard/box-java:latest`](https://hub.docker.com/r/konard/box-java/tags?name=latest) | [`latest-amd64`](https://hub.docker.com/r/konard/box-java/tags?name=latest-amd64) | [`latest-arm64`](https://hub.docker.com/r/konard/box-java/tags?name=latest-arm64) |
| Kotlin | [`konard/box-kotlin:latest`](https://hub.docker.com/r/konard/box-kotlin/tags?name=latest) | [`latest-amd64`](https://hub.docker.com/r/konard/box-kotlin/tags?name=latest-amd64) | [`latest-arm64`](https://hub.docker.com/r/konard/box-kotlin/tags?name=latest-arm64) |
| Ruby | [`konard/box-ruby:latest`](https://hub.docker.com/r/konard/box-ruby/tags?name=latest) | [`latest-amd64`](https://hub.docker.com/r/konard/box-ruby/tags?name=latest-amd64) | [`latest-arm64`](https://hub.docker.com/r/konard/box-ruby/tags?name=latest-arm64) |
| PHP | [`konard/box-php:latest`](https://hub.docker.com/r/konard/box-php/tags?name=latest) | [`latest-amd64`](https://hub.docker.com/r/konard/box-php/tags?name=latest-amd64) | [`latest-arm64`](https://hub.docker.com/r/konard/box-php/tags?name=latest-arm64) |
| Perl | [`konard/box-perl:latest`](https://hub.docker.com/r/konard/box-perl/tags?name=latest) | [`latest-amd64`](https://hub.docker.com/r/konard/box-perl/tags?name=latest-amd64) | [`latest-arm64`](https://hub.docker.com/r/konard/box-perl/tags?name=latest-arm64) |
| Swift | [`konard/box-swift:latest`](https://hub.docker.com/r/konard/box-swift/tags?name=latest) | [`latest-amd64`](https://hub.docker.com/r/konard/box-swift/tags?name=latest-amd64) | [`latest-arm64`](https://hub.docker.com/r/konard/box-swift/tags?name=latest-arm64) |
| Lean | [`konard/box-lean:latest`](https://hub.docker.com/r/konard/box-lean/tags?name=latest) | [`latest-amd64`](https://hub.docker.com/r/konard/box-lean/tags?name=latest-amd64) | [`latest-arm64`](https://hub.docker.com/r/konard/box-lean/tags?name=latest-arm64) |
| Rocq | [`konard/box-rocq:latest`](https://hub.docker.com/r/konard/box-rocq/tags?name=latest) | [`latest-amd64`](https://hub.docker.com/r/konard/box-rocq/tags?name=latest-amd64) | [`latest-arm64`](https://hub.docker.com/r/konard/box-rocq/tags?name=latest-arm64) |

### Docker Hub - dind-box (Docker-in-Docker variants, issue #80)

Each row below has the same toolchain as its non-dind sibling **plus** a working Docker Engine (Docker CLI + dockerd + containerd + Buildx + Compose v2). The default is nested Docker-in-Docker — each container has its own daemon, so `docker ps -a` from inside the container only lists containers it created. See the [security model](#docker-in-docker-security-model) section below and [docs/case-studies/issue-80](docs/case-studies/issue-80/CASE-STUDY.md).

| Image | Multi-arch | AMD64 | ARM64 |
|-------|------------|-------|-------|
| Full + dind | [`konard/box-dind:latest`](https://hub.docker.com/r/konard/box-dind/tags?name=latest) | [`latest-amd64`](https://hub.docker.com/r/konard/box-dind/tags?name=latest-amd64) | [`latest-arm64`](https://hub.docker.com/r/konard/box-dind/tags?name=latest-arm64) |
| Essentials + dind | [`konard/box-essentials-dind:latest`](https://hub.docker.com/r/konard/box-essentials-dind/tags?name=latest) | [`latest-amd64`](https://hub.docker.com/r/konard/box-essentials-dind/tags?name=latest-amd64) | [`latest-arm64`](https://hub.docker.com/r/konard/box-essentials-dind/tags?name=latest-arm64) |
| JS + dind | [`konard/box-js-dind:latest`](https://hub.docker.com/r/konard/box-js-dind/tags?name=latest) | [`latest-amd64`](https://hub.docker.com/r/konard/box-js-dind/tags?name=latest-amd64) | [`latest-arm64`](https://hub.docker.com/r/konard/box-js-dind/tags?name=latest-arm64) |
| Python + dind | [`konard/box-python-dind:latest`](https://hub.docker.com/r/konard/box-python-dind/tags?name=latest) | [`latest-amd64`](https://hub.docker.com/r/konard/box-python-dind/tags?name=latest-amd64) | [`latest-arm64`](https://hub.docker.com/r/konard/box-python-dind/tags?name=latest-arm64) |
| Go + dind | [`konard/box-go-dind:latest`](https://hub.docker.com/r/konard/box-go-dind/tags?name=latest) | [`latest-amd64`](https://hub.docker.com/r/konard/box-go-dind/tags?name=latest-amd64) | [`latest-arm64`](https://hub.docker.com/r/konard/box-go-dind/tags?name=latest-arm64) |
| Rust + dind | [`konard/box-rust-dind:latest`](https://hub.docker.com/r/konard/box-rust-dind/tags?name=latest) | [`latest-amd64`](https://hub.docker.com/r/konard/box-rust-dind/tags?name=latest-amd64) | [`latest-arm64`](https://hub.docker.com/r/konard/box-rust-dind/tags?name=latest-arm64) |
| Java + dind | [`konard/box-java-dind:latest`](https://hub.docker.com/r/konard/box-java-dind/tags?name=latest) | [`latest-amd64`](https://hub.docker.com/r/konard/box-java-dind/tags?name=latest-amd64) | [`latest-arm64`](https://hub.docker.com/r/konard/box-java-dind/tags?name=latest-arm64) |
| Kotlin + dind | [`konard/box-kotlin-dind:latest`](https://hub.docker.com/r/konard/box-kotlin-dind/tags?name=latest) | [`latest-amd64`](https://hub.docker.com/r/konard/box-kotlin-dind/tags?name=latest-amd64) | [`latest-arm64`](https://hub.docker.com/r/konard/box-kotlin-dind/tags?name=latest-arm64) |
| Ruby + dind | [`konard/box-ruby-dind:latest`](https://hub.docker.com/r/konard/box-ruby-dind/tags?name=latest) | [`latest-amd64`](https://hub.docker.com/r/konard/box-ruby-dind/tags?name=latest-amd64) | [`latest-arm64`](https://hub.docker.com/r/konard/box-ruby-dind/tags?name=latest-arm64) |
| PHP + dind | [`konard/box-php-dind:latest`](https://hub.docker.com/r/konard/box-php-dind/tags?name=latest) | [`latest-amd64`](https://hub.docker.com/r/konard/box-php-dind/tags?name=latest-amd64) | [`latest-arm64`](https://hub.docker.com/r/konard/box-php-dind/tags?name=latest-arm64) |
| Perl + dind | [`konard/box-perl-dind:latest`](https://hub.docker.com/r/konard/box-perl-dind/tags?name=latest) | [`latest-amd64`](https://hub.docker.com/r/konard/box-perl-dind/tags?name=latest-amd64) | [`latest-arm64`](https://hub.docker.com/r/konard/box-perl-dind/tags?name=latest-arm64) |
| Swift + dind | [`konard/box-swift-dind:latest`](https://hub.docker.com/r/konard/box-swift-dind/tags?name=latest) | [`latest-amd64`](https://hub.docker.com/r/konard/box-swift-dind/tags?name=latest-amd64) | [`latest-arm64`](https://hub.docker.com/r/konard/box-swift-dind/tags?name=latest-arm64) |
| Lean + dind | [`konard/box-lean-dind:latest`](https://hub.docker.com/r/konard/box-lean-dind/tags?name=latest) | [`latest-amd64`](https://hub.docker.com/r/konard/box-lean-dind/tags?name=latest-amd64) | [`latest-arm64`](https://hub.docker.com/r/konard/box-lean-dind/tags?name=latest-arm64) |
| Rocq + dind | [`konard/box-rocq-dind:latest`](https://hub.docker.com/r/konard/box-rocq-dind/tags?name=latest) | [`latest-amd64`](https://hub.docker.com/r/konard/box-rocq-dind/tags?name=latest-amd64) | [`latest-arm64`](https://hub.docker.com/r/konard/box-rocq-dind/tags?name=latest-arm64) |

### GitHub Container Registry - Combo Boxes

| Image | Multi-arch | AMD64 | ARM64 |
|-------|------------|-------|-------|
| Full Box | [`ghcr.io/link-foundation/box:latest`](https://github.com/link-foundation/box/pkgs/container/box?tag=latest) | [`latest-amd64`](https://github.com/link-foundation/box/pkgs/container/box?tag=latest-amd64) | [`latest-arm64`](https://github.com/link-foundation/box/pkgs/container/box?tag=latest-arm64) |
| Essentials | [`ghcr.io/link-foundation/box-essentials:latest`](https://github.com/link-foundation/box/pkgs/container/box-essentials?tag=latest) | [`latest-amd64`](https://github.com/link-foundation/box/pkgs/container/box-essentials?tag=latest-amd64) | [`latest-arm64`](https://github.com/link-foundation/box/pkgs/container/box-essentials?tag=latest-arm64) |
| JS | [`ghcr.io/link-foundation/box-js:latest`](https://github.com/link-foundation/box/pkgs/container/box-js?tag=latest) | [`latest-amd64`](https://github.com/link-foundation/box/pkgs/container/box-js?tag=latest-amd64) | [`latest-arm64`](https://github.com/link-foundation/box/pkgs/container/box-js?tag=latest-arm64) |

### GitHub Container Registry - Language Boxes

| Language | Multi-arch | AMD64 | ARM64 |
|----------|------------|-------|-------|
| Python | [`ghcr.io/link-foundation/box-python:latest`](https://github.com/link-foundation/box/pkgs/container/box-python?tag=latest) | [`latest-amd64`](https://github.com/link-foundation/box/pkgs/container/box-python?tag=latest-amd64) | [`latest-arm64`](https://github.com/link-foundation/box/pkgs/container/box-python?tag=latest-arm64) |
| Go | [`ghcr.io/link-foundation/box-go:latest`](https://github.com/link-foundation/box/pkgs/container/box-go?tag=latest) | [`latest-amd64`](https://github.com/link-foundation/box/pkgs/container/box-go?tag=latest-amd64) | [`latest-arm64`](https://github.com/link-foundation/box/pkgs/container/box-go?tag=latest-arm64) |
| Rust | [`ghcr.io/link-foundation/box-rust:latest`](https://github.com/link-foundation/box/pkgs/container/box-rust?tag=latest) | [`latest-amd64`](https://github.com/link-foundation/box/pkgs/container/box-rust?tag=latest-amd64) | [`latest-arm64`](https://github.com/link-foundation/box/pkgs/container/box-rust?tag=latest-arm64) |
| Java | [`ghcr.io/link-foundation/box-java:latest`](https://github.com/link-foundation/box/pkgs/container/box-java?tag=latest) | [`latest-amd64`](https://github.com/link-foundation/box/pkgs/container/box-java?tag=latest-amd64) | [`latest-arm64`](https://github.com/link-foundation/box/pkgs/container/box-java?tag=latest-arm64) |
| Kotlin | [`ghcr.io/link-foundation/box-kotlin:latest`](https://github.com/link-foundation/box/pkgs/container/box-kotlin?tag=latest) | [`latest-amd64`](https://github.com/link-foundation/box/pkgs/container/box-kotlin?tag=latest-amd64) | [`latest-arm64`](https://github.com/link-foundation/box/pkgs/container/box-kotlin?tag=latest-arm64) |
| Ruby | [`ghcr.io/link-foundation/box-ruby:latest`](https://github.com/link-foundation/box/pkgs/container/box-ruby?tag=latest) | [`latest-amd64`](https://github.com/link-foundation/box/pkgs/container/box-ruby?tag=latest-amd64) | [`latest-arm64`](https://github.com/link-foundation/box/pkgs/container/box-ruby?tag=latest-arm64) |
| PHP | [`ghcr.io/link-foundation/box-php:latest`](https://github.com/link-foundation/box/pkgs/container/box-php?tag=latest) | [`latest-amd64`](https://github.com/link-foundation/box/pkgs/container/box-php?tag=latest-amd64) | [`latest-arm64`](https://github.com/link-foundation/box/pkgs/container/box-php?tag=latest-arm64) |
| Perl | [`ghcr.io/link-foundation/box-perl:latest`](https://github.com/link-foundation/box/pkgs/container/box-perl?tag=latest) | [`latest-amd64`](https://github.com/link-foundation/box/pkgs/container/box-perl?tag=latest-amd64) | [`latest-arm64`](https://github.com/link-foundation/box/pkgs/container/box-perl?tag=latest-arm64) |
| Swift | [`ghcr.io/link-foundation/box-swift:latest`](https://github.com/link-foundation/box/pkgs/container/box-swift?tag=latest) | [`latest-amd64`](https://github.com/link-foundation/box/pkgs/container/box-swift?tag=latest-amd64) | [`latest-arm64`](https://github.com/link-foundation/box/pkgs/container/box-swift?tag=latest-arm64) |
| Lean | [`ghcr.io/link-foundation/box-lean:latest`](https://github.com/link-foundation/box/pkgs/container/box-lean?tag=latest) | [`latest-amd64`](https://github.com/link-foundation/box/pkgs/container/box-lean?tag=latest-amd64) | [`latest-arm64`](https://github.com/link-foundation/box/pkgs/container/box-lean?tag=latest-arm64) |
| Rocq | [`ghcr.io/link-foundation/box-rocq:latest`](https://github.com/link-foundation/box/pkgs/container/box-rocq?tag=latest) | [`latest-amd64`](https://github.com/link-foundation/box/pkgs/container/box-rocq?tag=latest-amd64) | [`latest-arm64`](https://github.com/link-foundation/box/pkgs/container/box-rocq?tag=latest-arm64) |

### GitHub Container Registry - dind-box (Docker-in-Docker variants, issue #80)

| Image | Multi-arch | AMD64 | ARM64 |
|-------|------------|-------|-------|
| Full + dind | [`ghcr.io/link-foundation/box-dind:latest`](https://github.com/link-foundation/box/pkgs/container/box-dind?tag=latest) | [`latest-amd64`](https://github.com/link-foundation/box/pkgs/container/box-dind?tag=latest-amd64) | [`latest-arm64`](https://github.com/link-foundation/box/pkgs/container/box-dind?tag=latest-arm64) |
| Essentials + dind | [`ghcr.io/link-foundation/box-essentials-dind:latest`](https://github.com/link-foundation/box/pkgs/container/box-essentials-dind?tag=latest) | [`latest-amd64`](https://github.com/link-foundation/box/pkgs/container/box-essentials-dind?tag=latest-amd64) | [`latest-arm64`](https://github.com/link-foundation/box/pkgs/container/box-essentials-dind?tag=latest-arm64) |
| JS + dind | [`ghcr.io/link-foundation/box-js-dind:latest`](https://github.com/link-foundation/box/pkgs/container/box-js-dind?tag=latest) | [`latest-amd64`](https://github.com/link-foundation/box/pkgs/container/box-js-dind?tag=latest-amd64) | [`latest-arm64`](https://github.com/link-foundation/box/pkgs/container/box-js-dind?tag=latest-arm64) |
| Python + dind | [`ghcr.io/link-foundation/box-python-dind:latest`](https://github.com/link-foundation/box/pkgs/container/box-python-dind?tag=latest) | [`latest-amd64`](https://github.com/link-foundation/box/pkgs/container/box-python-dind?tag=latest-amd64) | [`latest-arm64`](https://github.com/link-foundation/box/pkgs/container/box-python-dind?tag=latest-arm64) |
| Go + dind | [`ghcr.io/link-foundation/box-go-dind:latest`](https://github.com/link-foundation/box/pkgs/container/box-go-dind?tag=latest) | [`latest-amd64`](https://github.com/link-foundation/box/pkgs/container/box-go-dind?tag=latest-amd64) | [`latest-arm64`](https://github.com/link-foundation/box/pkgs/container/box-go-dind?tag=latest-arm64) |
| Rust + dind | [`ghcr.io/link-foundation/box-rust-dind:latest`](https://github.com/link-foundation/box/pkgs/container/box-rust-dind?tag=latest) | [`latest-amd64`](https://github.com/link-foundation/box/pkgs/container/box-rust-dind?tag=latest-amd64) | [`latest-arm64`](https://github.com/link-foundation/box/pkgs/container/box-rust-dind?tag=latest-arm64) |
| Java + dind | [`ghcr.io/link-foundation/box-java-dind:latest`](https://github.com/link-foundation/box/pkgs/container/box-java-dind?tag=latest) | [`latest-amd64`](https://github.com/link-foundation/box/pkgs/container/box-java-dind?tag=latest-amd64) | [`latest-arm64`](https://github.com/link-foundation/box/pkgs/container/box-java-dind?tag=latest-arm64) |
| Kotlin + dind | [`ghcr.io/link-foundation/box-kotlin-dind:latest`](https://github.com/link-foundation/box/pkgs/container/box-kotlin-dind?tag=latest) | [`latest-amd64`](https://github.com/link-foundation/box/pkgs/container/box-kotlin-dind?tag=latest-amd64) | [`latest-arm64`](https://github.com/link-foundation/box/pkgs/container/box-kotlin-dind?tag=latest-arm64) |
| Ruby + dind | [`ghcr.io/link-foundation/box-ruby-dind:latest`](https://github.com/link-foundation/box/pkgs/container/box-ruby-dind?tag=latest) | [`latest-amd64`](https://github.com/link-foundation/box/pkgs/container/box-ruby-dind?tag=latest-amd64) | [`latest-arm64`](https://github.com/link-foundation/box/pkgs/container/box-ruby-dind?tag=latest-arm64) |
| PHP + dind | [`ghcr.io/link-foundation/box-php-dind:latest`](https://github.com/link-foundation/box/pkgs/container/box-php-dind?tag=latest) | [`latest-amd64`](https://github.com/link-foundation/box/pkgs/container/box-php-dind?tag=latest-amd64) | [`latest-arm64`](https://github.com/link-foundation/box/pkgs/container/box-php-dind?tag=latest-arm64) |
| Perl + dind | [`ghcr.io/link-foundation/box-perl-dind:latest`](https://github.com/link-foundation/box/pkgs/container/box-perl-dind?tag=latest) | [`latest-amd64`](https://github.com/link-foundation/box/pkgs/container/box-perl-dind?tag=latest-amd64) | [`latest-arm64`](https://github.com/link-foundation/box/pkgs/container/box-perl-dind?tag=latest-arm64) |
| Swift + dind | [`ghcr.io/link-foundation/box-swift-dind:latest`](https://github.com/link-foundation/box/pkgs/container/box-swift-dind?tag=latest) | [`latest-amd64`](https://github.com/link-foundation/box/pkgs/container/box-swift-dind?tag=latest-amd64) | [`latest-arm64`](https://github.com/link-foundation/box/pkgs/container/box-swift-dind?tag=latest-arm64) |
| Lean + dind | [`ghcr.io/link-foundation/box-lean-dind:latest`](https://github.com/link-foundation/box/pkgs/container/box-lean-dind?tag=latest) | [`latest-amd64`](https://github.com/link-foundation/box/pkgs/container/box-lean-dind?tag=latest-amd64) | [`latest-arm64`](https://github.com/link-foundation/box/pkgs/container/box-lean-dind?tag=latest-arm64) |
| Rocq + dind | [`ghcr.io/link-foundation/box-rocq-dind:latest`](https://github.com/link-foundation/box/pkgs/container/box-rocq-dind?tag=latest) | [`latest-amd64`](https://github.com/link-foundation/box/pkgs/container/box-rocq-dind?tag=latest-amd64) | [`latest-arm64`](https://github.com/link-foundation/box/pkgs/container/box-rocq-dind?tag=latest-arm64) |

### Docker-in-Docker security model

> ⚠️ **dind-box variants ship a Docker daemon and require elevated privileges to run.**
>
> - **Default invocation:** `docker run --privileged konard/box-dind` — the inner `dockerd` needs `CAP_SYS_ADMIN` and access to the kernel's overlay/namespace machinery. Each container has its own daemon, so `docker ps -a` from inside lists only that container's children.
> - **Recommended secure invocation:** [`docker run --runtime=sysbox-runc konard/box-dind`](https://github.com/nestybox/sysbox) — Sysbox is a drop-in OCI runtime that runs system containers without `--privileged` and without exposing host devices.
> - **Do NOT bind-mount `/var/run/docker.sock`.** That gives the container root on the host ([Quarkslab](https://blog.quarkslab.com/why-is-exposing-the-docker-socket-a-really-bad-idea.html), [OWASP](https://cheatsheetseries.owasp.org/cheatsheets/Docker_Security_Cheat_Sheet.html)) and breaks the per-box `docker ps` scoping property.
> - **Storage:** the inner daemon writes to `/var/lib/docker` inside the container by default. For persistence, mount a volume: `-v box-dind-data:/var/lib/docker`.

See [docs/case-studies/issue-80/CASE-STUDY.md](docs/case-studies/issue-80/CASE-STUDY.md) for the full design and threat model.

## Usage

### Quick Start

Pull multi-arch (auto-selects your platform):
```bash
docker pull konard/box:latest
```

Pull specific architecture:
```bash
# AMD64
docker pull konard/box:latest-amd64

# ARM64 (Apple Silicon, Raspberry Pi, etc.)
docker pull konard/box:latest-arm64
```

Pull from GHCR:
```bash
docker pull ghcr.io/link-foundation/box:latest
```

### Run interactively

```bash
docker run -it ghcr.io/link-foundation/box:latest
```

### Use as base image

```dockerfile
FROM ghcr.io/link-foundation/box:latest

# Your additional setup here
COPY . /home/box
RUN npm install
```

### Build locally

```bash
git clone https://github.com/link-foundation/box.git
cd box
docker build -t box .
```

## Architecture Support

The image is built for both `linux/amd64` and `linux/arm64` architectures.

### CI/CD Build Requirements

**IMPORTANT: ARM64 builds MUST use native ARM64 runners.**

| Architecture | Runner | Build Time |
|-------------|--------|------------|
| `linux/amd64` | `ubuntu-latest` | ~5-10 minutes |
| `linux/arm64` | `ubuntu-24.04-arm` (native) | ~30-60 minutes |

Native ARM64 runners provide optimal build performance for compilation-heavy workloads. Emulation would incur a 10-30x performance penalty.

For detailed analysis, see [Case Study: Issue #7](docs/case-studies/issue-7/README.md).

## Environment

The container runs as the `box` user with home directory at `/home/box`. All language runtimes are installed in user-local directories under `/home/box`:

- Node.js: `~/.nvm`
- Python: `~/.pyenv`
- Go: `~/.go`
- Rust: `~/.cargo`
- Java/Kotlin: `~/.sdkman`
- Ruby: `~/.rbenv`
- Swift: `~/.swift`
- Lean: `~/.elan`
- Perl: `~/.perl5`
- OCaml/Rocq: `~/.opam`

<!-- COMPONENT_SIZES_START -->
## Component Sizes

_Last updated: 2026-04-06T08:34:59Z_

**Total installation size: 7280.0 MB**

| Component | Category | Size (MB) |
|-----------|----------|-----------|
| Swift 6.x | Runtime | 2655.24 |
| Opam + Rocq/Coq | Runtime | 1017.0 |
| Rust (via rustup) | Runtime | 647.44 |
| SDKMAN + Java 21 | Runtime | 579.09 |
| .NET SDK 8.0 | Runtime | 504.93 |
| Pyenv + Python (latest) | Runtime | 387.94 |
| Perlbrew + Perl (latest) | Runtime | 364.98 |
| Go (latest) | Runtime | 281.75 |
| NVM + Node.js 20 | Runtime | 230.95 |
| Kotlin (via SDKMAN) | Runtime | 173.97 |
| rbenv + Ruby (latest) | Runtime | 165.15 |
| Deno | Runtime | 123.64 |
| R Language | Runtime | 121.13 |
| Bun | Runtime | 99.68 |
| PHP 8.3 (via Homebrew) | Runtime | 55.47 |
| Lean (via elan) | Runtime | 13.23 |
| C/C++ Tools (CMake, Clang, LLVM, LLD) | Build Tools | 58.83 |
| Assembly Tools (NASM, FASM) | Build Tools | 3.92 |
| GitLab CLI | Development Tools | 30.06 |
| gh-setup-git-identity | Development Tools | 4.61 |
| glab-setup-git-identity | Development Tools | 4.48 |
| GitHub CLI | Development Tools | 0.54 |
| Homebrew | Package Manager | 179.75 |
| Python Build Dependencies | Dependencies | 44.24 |
| Bubblewrap | Dependencies | 0.18 |
| Ruby Build Dependencies | Dependencies | 0.0 |
| Essential Tools | System | 0.75 |

_Note: Sizes are measured after cleanup and may vary based on system state and package versions._

<!-- COMPONENT_SIZES_END -->

## Releasing

Releases are produced by `.github/workflows/release.yml`, which builds and pushes images to **both** the GitHub Container Registry (GHCR) and Docker Hub.

The workflow uses two sets of credentials:

| Registry | Credential | Source |
|---|---|---|
| `ghcr.io` | `GITHUB_TOKEN` | Auto-provisioned by GitHub Actions; no rotation required. |
| `docker.io` | `secrets.DOCKERHUB_USERNAME` + `secrets.DOCKERHUB_TOKEN` | Configured at https://github.com/link-foundation/box/settings/secrets/actions. The `DOCKERHUB_TOKEN` is a Docker Hub Personal Access Token (PAT). |

### Rotating the Docker Hub PAT

Docker Hub PATs have an immutable expiry (set when the token is created) and **cannot be renewed in place** — they must be re-issued and re-saved as the `DOCKERHUB_TOKEN` secret. When the token expires, the release workflow will continue to push to GHCR (since issue #82) but will emit a `::warning` annotation on every build job and skip the Docker Hub portion of the release.

Rotation runbook:

1. Sign in to Docker Hub at https://hub.docker.com/settings/security
2. Click **New Access Token**, scope it to `Read, Write, Delete` for the `konard/box*` repositories, set an expiry that fits the rotation policy (e.g. 90 days), and copy the token.
3. Open https://github.com/link-foundation/box/settings/secrets/actions and update `DOCKERHUB_TOKEN`.
4. Re-run the most recent failed release run, or push an empty commit, or trigger `workflow_dispatch` on `Build and Release Docker Image` with `release_mode=release-only` to re-publish the current `VERSION` to Docker Hub.

See [`docs/case-studies/issue-82/CASE-STUDY.md`](docs/case-studies/issue-82/CASE-STUDY.md) for the incident that introduced this hardening.

## Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture and design decisions
- [REQUIREMENTS.md](REQUIREMENTS.md) - Project requirements and constraints
- [docs/case-studies/](docs/case-studies/) - Case studies and incident analysis

## License

MIT License - see [LICENSE](LICENSE) for details.

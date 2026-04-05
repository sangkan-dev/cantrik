# Contributing to Cantrik

Terima kasih untuk minat berkontribusi ke Cantrik! 🎉

## Code of Conduct

Kami berkomitmen untuk menjaga komunitas yang welcoming dan inclusive. Semua kontributor harus mengikuti [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/).

---

## Getting Started

### Prerequisites
- **Rust 1.70+** ([install rustup](https://rustup.rs/))
- **Git**
- **Node.js 20+** (hanya jika Anda mengubah hub di `apps/cantrik-site/`)
- **Python 3.8+** (untuk PRD docs generation, optional)

### Setup Development Environment

```bash
# Clone repository
git clone https://github.com/sangkan-dev/cantrik.git
cd cantrik

# Build dan test
cargo build
cargo test

# Run pre-commit checks
.githooks/pre-commit
```

---

## Network surfaces (enterprise / air-gapped audit)

Use this as a checklist when hardening or documenting deployments—not every path is blocked by `[llm] offline` / `CANTRIK_OFFLINE`.

| Area | Mechanism | Notes |
|------|-----------|--------|
| LLM chat / streaming | `reqwest` to cloud APIs or Ollama | Offline mode restricts the **LLM chain** to loopback Ollama only. |
| `cantrik fetch`, `cantrik web` | `reqwest` after `--approve` | **Blocked** when offline mode is on. |
| Indexing / embeddings | Ollama (and LanceDB local) | Point Ollama at loopback for fully local embeddings. |
| MCP client | Child stdio / configured servers | May reach network depending on server; review `providers.toml` / MCP config. |
| `cantrik mcp call` | Spawns MCP server process; tools may use network | Same as MCP — audit each `[[mcp_client.servers]]` entry. |
| Background jobs + webhooks | Optional HTTP POST | `[background].webhook_url` when a job enters `waiting_approval` (skipped when offline mode is on). |
| Git workflow / PR | `gh` or HTTPS to GitHub | `cantrik pr`, `workspace branch`, etc.; uses network when not `none`. |
| Browse / fetch docs tools | HTTP when agent uses `browse_page` / `fetch_docs` / `web_search` | Refused when `[llm].offline` / `CANTRIK_OFFLINE` (same as `cantrik fetch` / `web`). |
| Voice (`cantrik listen`) | POST to local Ollama `/api/transcribe` | Usually loopback; still HTTP. |
| Plugins (Lua/WASM) | Sandboxed but can expose tools | Review plugin code and capabilities. |

### Enterprise release checklist

Sebelum rilis atau deploy ke lingkungan ketat, tinjau ulang (bukan sekali jalan otomatis):

- [ ] Daftar MCP: setiap `[[mcp_client.servers]]` — proses apa yang di-spawn, apakah tool-nya bisa keluar jaringan.
- [ ] Plugin (Lua/WASM): sumber tepercaya; kemampuan tool yang diekspos.
- [ ] `[background].webhook_url`: URL tujuan, TLS, siapa yang membaca payload approval.
- [ ] Embedding / Ollama: host loopback vs remote; data apa yang dikirim.
- [ ] `CANTRIK_OFFLINE` / `[llm].offline`: sudah sesuai kebijakan untuk fetch, agent tools, dan webhook (lihat tabel di atas).

**Template issue:** gunakan [Release / air-gap audit](.github/ISSUE_TEMPLATE/release_audit.md) saat menyiapkan rilis atau deploy enterprise.

### Distribusi (maintainer, per tag rilis)

- **pacman:** setelah tag ada, isi `sha256sums` di [`packaging/arch/PKGBUILD`](packaging/arch/PKGBUILD) dari artefak sumber (mis. `curl -sL URL | sha256sum` atau `updpkgsums`). `SKIP` hanya untuk iterasi lokal.
- **winget:** perbarui `InstallerSha256` di [`packaging/winget/Sangkan.Cantrik.yaml`](packaging/winget/Sangkan.Cantrik.yaml) agar cocok dengan binary release; CI memvalidasi sintaks YAML ([`winget-validate`](.github/workflows/winget-validate.yml)).
- **Nix:** derivasi `nix build` penuh belum disediakan; gunakan devShell di [`packaging/nix/README.md`](packaging/nix/README.md) + `cargo install --path crates/cantrik-cli` sampai iterasi flake khusus.

### Registry recipes (komunitas)

Untuk menambah entri di [`apps/cantrik-site/static/registry/recipes.json`](apps/cantrik-site/static/registry/recipes.json):

1. Fork + branch, edit JSON: setiap objek di `recipes` wajib punya string non-kosong: `id`, `title`, `init_template` (dan disarankan `description`).
2. Jalankan `python3 scripts/validate-recipes-registry.py apps/cantrik-site/static/registry/recipes.json` sebelum PR.
3. PR kecil satu tema (satu batch recipe atau satu perubahan terkait) memudahkan review.

**Kurasi editorial (maintainer):** merge ke registry utama hanya oleh maintainer `sangkan-dev/cantrik` setelah review PR. Tolak atau minta revisi jika: spam / promosi, duplikat `id` atau overlap besar dengan entri ada, `init_template` tidak cocok template [`cantrik init`](crates/cantrik-cli/), atau konten menyesatkan. Field opsional boolean `verified: true` hanya boleh ditambahkan maintainer setelah entri diuji (template init berjalan sesuai deskripsi); kontributor umum tidak perlu mengisi `verified`.

### Enterprise sandbox (gVisor / runsc)

- Set `sandbox.level = "container"` **hanya** jika operator siap: butuh biner `runsc` dan kebijakan host.
- Set env `CANTRIK_RUNSC_BIN` ke path `runsc`. Opsional: `CANTRIK_RUNSC_RUN_ARGS` — token dipisah spasi yang disisipkan setelah `runsc run` (contoh: `--network=none` jika didukung setup Anda).
- Tanpa env tersebut, level `container` gagal dengan pesan jelas (lebih aman daripada mengeksekusi tanpa isolasi yang dimaksud).
- **CI:** runner GitHub Actions default tidak menyediakan `runsc` atau izin privileged; gunakan runner self-hosted atau abaikan job khusus sampai kebijakan infra jelas. Validasi lokal dengan `CANTRIK_RUNSC_BIN` tetap menjadi tanggung jawab operator. Workflow opsional (manual): [`.github/workflows/runsc-sandbox-smoke.yml`](.github/workflows/runsc-sandbox-smoke.yml) — set variabel repo `CANTRIK_RUNSC_SELF_HOSTED=true` hanya jika runner self-hosted dengan `runsc` tersedia.

### Desktop tray (Tauri) — file flag approval

Companion [`apps/cantrik-tauri`](apps/cantrik-tauri/) memantau file yang sama dengan daemon/tray Rust:

- Default: `data_local_dir()/cantrik/approval-pending.flag` (biasanya `~/.local/share/cantrik/approval-pending.flag` pada Linux).
- Override penuh: env `CANTRIK_APPROVAL_FLAG_PATH` (path absolut atau relatif ke cwd proses tray).
- Agar selaras dengan merge config CLI: set `CANTRIK_PROJECT_ROOT` ke root repo; Tauri membaca `~/.config/cantrik/config.toml` lalu `.cantrik/cantrik.toml` untuk field `[background].approval_flag_path` (proyek menimpa global).

### Self-improvement (safe profile)

Menjalankan Cantrik pada **repo Cantrik sendiri** untuk saran perbaikan:

- Gunakan sandbox `restricted`, batasi token/konteks di config, jangan `--approve` massal tanpa review diff.
- Anggap biaya API dan risiko prompt injection dari konten repo; dokumentasikan asumsi di issue/PR.
- Otomatisasi penuh “self-rewrite” tetap di backlog hingga ada gate produk (tes, approval, rollback).
- **Dry-run skrip:** [`scripts/self-improve-dry-run.sh`](scripts/self-improve-dry-run.sh) menjalankan satu `cantrik ask` read-only pada repo (tanpa `--approve`); gunakan sebagai langkah manual sebelum otomasi patch.
- **Gate sebelum otomasi patch (MVP):** wajibkan dry-run + review manusia pada diff; batasi konteks/token di config; otomasi PR hanya dari **fork** atau branch eksperimen; tidak ada loop tanpa henti tanpa human-in-the-loop.
- **CI opsional (fork / manual):** [`.github/workflows/self-improve-gate.yml`](.github/workflows/self-improve-gate.yml) — `workflow_dispatch`: gate `cargo test --workspace`; centang input `run_llm_dry_run` untuk menjalankan skrip dry-run setelah build (butuh kunci provider di runner).

### SWE constrained workflow (manual checklist)

Alur terbatas untuk satu issue publik + satu repo lokal:

1. Set `ISSUE_URL` dan jalankan [`scripts/swe-fix-demo.sh`](scripts/swe-fix-demo.sh) **atau** `cantrik fix "$URL" --approve --fetch` lalu tinjau HTML di stdout.
2. Opsional: `--run-agents` (batas waktu: env `CANTRIK_FIX_AGENT_TIMEOUT_SEC`, default 900).
3. Opsional: `--experiment` — masih membutuhkan `--approve`; menjalankan mode experiment (writes + tes + revert) — review diff sebelum commit.
4. `cantrik pr create --approve` atau alur Git manual; tidak ada jaminan “high reliability” sampai ada tes integrasi produk.

**Kebijakan / QA produk (rantai `fix`):** flag `--run-agents` / `--run-experiment` wajib dipasangkan dengan `--approve` dan `--fetch`; invariant ini dites di unit test (`fix_cmd::validate_fix_flags`). Tes integrasi **tanpa LLM**: wiremock + HTML dari berkas terversion [`tests/fixtures/cantrik-fix-issue-sample.html`](tests/fixtures/cantrik-fix-issue-sample.html) (`fix_approve_fetch_reaches_fixture_file`) + workflow [`.github/workflows/swe-e2e-smoke.yml`](.github/workflows/swe-e2e-smoke.yml). **Opsional (jaringan):** set env `CANTRIK_FIX_E2E_HTTP_URL` ke URL statis terpin (mis. raw GitHub ke file fixture) lalu jalankan `cargo test -p cantrik-cli fix_optional_pinned_http_url_from_env` — tidak diwajibkan di CI utama (bisa rapuh bila URL berubah).

**Definisi “high reliability” (SWE otonom penuh):** lihat template issue [SWE E2E reliability](.github/ISSUE_TEMPLATE/swe_e2e_reliability.md) — minimal satu skenario E2E terotomatisasi sebelum mengklaim item backlog terkait selesai.

### Phase 5 triage (contributors)

- **Tree-sitter:** lihat [`docs/tree-sitter-language-extensions.md`](docs/tree-sitter-language-extensions.md) (kompatibilitas grammar vs versi `tree-sitter` workspace); satu bahasa per PR.
- **Sandbox enterprise (gVisor / Firecracker):** titik masuk ada di `crates/cantrik-core/src/tool_system/sandbox.rs`; butuh desain admin + CI khusus.
- **Hybrid SSH / cloud executor:** RFC [`docs/rfc-hybrid-ssh-executor.md`](docs/rfc-hybrid-ssh-executor.md); MVP CLI: `cantrik exec --remote` + `[remote_exec]`. **Sync:** `cantrik sync` mencetak perintah `rsync` (mode `--dry-run` bawaan); `cantrik sync --approve` menjalankan rsync setelah review. Set `[remote_exec].sync_remote_dir`. Manual `rsync`/`scp` tetap didukung untuk alur kustom.
- **Benchmark SWE-bench / Terminal-Bench:** skrip baseline [`scripts/phase5-smoke.sh`](scripts/phase5-smoke.sh) (quality gates + hook `CANTRIK_BENCH_HARNESS=1`); demo alur terbatas [`scripts/swe-fix-demo.sh`](scripts/swe-fix-demo.sh) dengan `ISSUE_URL=…`.
- **`cantrik fix` / agents:** `cantrik fix URL --approve --fetch --run-agents` (+ timeout `CANTRIK_FIX_AGENT_TIMEOUT_SEC`); `--run-experiment` merantai mode experiment; `cantrik agents "…" --reflect` = satu putaran reviewer LLM.
- **Agent harness artefak:** `cantrik status --write-harness-summary` menulis `.cantrik/session-harness-summary.json` (payload job + `generated_at_unix`) untuk di-attach ke CI atau dashboard eksternal.

---

## Development Workflow

### 1. Choose or Create an Issue

- Cek [open issues](https://github.com/sangkan-dev/cantrik/issues)
- Pilih yang sesuai skill atau buat issue baru untuk feature/bug
- Tag dengan `good first issue` kalau baru pertama kali

### 2. Create Feature Branch

```bash
git checkout -b sprint-N/feature-name

# Contoh:
git checkout -b sprint-2/add-ask-subcommand
git checkout -b bugfix/config-parser-panic
```

### 3. Code dengan Standards

#### Rust Code Standards
- **Format:** `cargo fmt` (automatic)
- **Lint:** `cargo clippy -- -D warnings` (zero warnings policy)
- **Tests:** Tambah unit test untuk logic non-trivial
- **Error Handling:** Gunakan `Result<T, E>` dan `thiserror`
- **Documentation:** Doc comments untuk public types/functions

#### Contoh Code Style

```rust
/// Load configuration from disk with 2-tier precedence.
/// 
/// Priority: project `.cantrik/` > global `~/.config/cantrik/` > defaults
pub fn load_merged_config(cwd: &Path) -> Result<AppConfig, ConfigError> {
    let global = read_if_exists(global_config_path())?;
    let project = read_if_exists(cwd.join(".cantrik/cantrik.toml"))?;
    
    let mut config = global.unwrap_or_default();
    if let Some(project_cfg) = project {
        config.merge(project_cfg);
    }
    
    Ok(config)
}

#[cfg(test)]
mod tests {
    #[test]
    fn project_config_overrides_global() {
        // test implementation
    }
}
```

### 4. Local Quality Gates

Run sebelum commit:

```bash
# Format code
cargo fmt

# Type check
cargo check --workspace

# Test
cargo test

# Lint (no warnings allowed)
cargo clippy -- -D warnings
```

Atau gunakan pre-commit hook:
```bash
.githooks/pre-commit
```

### 5. Commit Message

Ikuti [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:** `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`

**Contoh:**
```
feat(cli): add ask subcommand with streaming support

- Implement clap subcommand parser for 'ask' verb
- Wire message history from config
- Add streaming response handler to stdout
- Tests cover basic ask + empty prompt error cases

Closes #42
```

### 6. Push & Create Pull Request

```bash
git push origin sprint-N/feature-name
```

Buka PR di GitHub dengan:
- **Title:** Clear, sesuai conventional commits
- **Description:** Jelaskan problem, solution, dan testing
- **Link issue:** "Closes #123" atau "Fixes #456"
- **Checklist:** Pastikan semua checks di PR template

#### PR Checklist

```markdown
- [ ] Code passes `cargo fmt --check`
- [ ] Code passes `cargo clippy -- -D warnings`
- [ ] Tests added/updated for new logic
- [ ] Test pass locally (`cargo test`)
- [ ] Documentation updated (README, doc comments, TASK.md)
- [ ] TASK.md status updated if applicable
- [ ] No unrelated changes included
```

### 7. Code Review

- Maintainer akan review code Anda
- Respond to feedback dengan explanations atau fixes
- Approve = ready to merge

---

## Sprint System & Task Tracking

Kami pakai 2-week sprint dengan breaking down tasks di [TASK.md](../TASK.md).

### Task Lifecycle

```
[ ] → belum dikerjakan (not started)
[/] → sedang berjalan (in progress)
[x] → selesai (completed)
```

### Sprint Workflow

1. **Plan:** Define tasks > assign > estimate time
2. **Develop:** Branch → code → test → PR
3. **Review:** Code review → address feedback
4. **Merge:** Squash merge to main (optionally)
5. **Verify:** TASK.md status → `[x]`

**Current Sprint:** See [TASK.md](../TASK.md)

---

## Architecture & Design

### Multi-Crate Structure

```
cantrik-core/        → Library: config, providers, indexing
cantrik-cli/         → Binary: CLI entrypoint, command dispatch
```

### Key Principles

1. **Modular:** Setiap fitur independent crate kalau besar
2. **Async-first:** Gunakan tokio untuk I/O
3. **Type-safe:** Trust compiler, minimize runtime errors
4. **Tested:** Unit tests + doc comments for clarity

### Configuration System

```toml
# Global: ~/.config/cantrik/config.toml
# Project: .cantrik/cantrik.toml (overrides global)
# Defaults: hard-coded in code

[llm]
provider = "anthropic"
model = "claude-3-sonnet"

[ui]
theme = "dark"
```

---

## Testing

### Unit Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test --lib config_overrides

# With output
cargo test -- --nocapture

# Single-threaded (useful for debugging)
cargo test -- --test-threads=1
```

### Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_load_config_merges_global_and_project() {
        // arrange
        // act
        // assert
    }
}
```

---

## Documentation

### Code Documentation

```rust
/// Brief description.
///
/// Longer explanation if needed.
///
/// # Arguments
/// * `path` - Path to config file
///
/// # Returns
/// Config loaded and merged with defaults
///
/// # Errors
/// Returns ConfigError if file cannot be read
pub fn load_config(path: &Path) -> Result<AppConfig, ConfigError> {
    // ...
}
```

### Markdown Documentation

- Keep README up-to-date with new features
- Add ARCHITECTURE.md when design changes significantly
- Update TASK.md as you complete tasks

---

## Communication

### Questions & Help

- **GitHub Issues:** Bug reports, feature requests
- **GitHub Discussions:** General questions (if enabled)
- **Code Review:** Questions during PR review process

### Reporting Issues

Include:
- Clear title + description
- Steps to reproduce
- Expected vs actual behavior
- Rust version (`rustc --version`)
- Cargo version (`cargo --version`)

Example:
```markdown
**Title:** Config parser panics on invalid TOML

**Steps to reproduce:**
1. Create invalid [cantrik/config.toml]({path to file})
2. Run `cantrik ask "hello"`

**Expected:** Graceful error message
**Actual:** Panic with backtrace
```

---

## Review Criteria

Maintainers akan check:

✅ **Code Quality**
- Follows Rust conventions (rustfmt, clippy green)
- Type-safe, no unwrap in production paths
- Tests included
- Doc comments present

✅ **Design**
- Aligns with architecture
- No breaking changes without discussion
- Dependencies justified
- Performance-conscious

✅ **Documentation**
- README updated if user-facing
- TASK.md status updated
- PR description clear
- Commit messages conventional

---

## Merging

Once approved by maintainer:
- PR will be merged to `main` (or `develop` as configured)
- Commit message preserved
- TASK.md auto-updated

---

## Hub web (`apps/cantrik-site`)

Situs statis SvelteKit (nuansa Sangkan / Ancient Cybernetics). CI: workflow `cantrik-site.yml`.

```bash
cd apps/cantrik-site
npm ci
npm run check
npm run lint
npm run build
```

## Release binary (GitHub)

Push an annotated tag `v0.x.y` — workflow `release.yml` membangun `cantrik` (release, Linux) dan mengunggahnya ke GitHub Releases. Dokumentasikan checksum/manual verify di catatan rilis bila perlu.

---

## Questions?

- Open an issue with `question` label
- Check [README.md](../README.md) and [TASK.md](../TASK.md)
- Review existing PRs for patterns

---

**Thank you for contributing to Cantrik!** 🙏

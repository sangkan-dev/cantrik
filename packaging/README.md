# Packaging checklist (maintainers)

## Release binary (GitHub Actions)

Workflow [`.github/workflows/release.yml`](../.github/workflows/release.yml) builds with:

`cargo build --release -p cantrik-cli`

Artifact: `target/release/cantrik`.

## Arch Linux (`arch/PKGBUILD`)

1. Set `pkgver` to match the git tag (without `v` if you use `refs/tags/v$pkgver` as in the template).
2. Replace `sha256sums=('SKIP')` with the real digest after the tarball exists:
   - `curl -sL "https://github.com/sangkan-dev/cantrik/archive/refs/tags/v${pkgver}.tar.gz" | sha256sum`
   - or run `updpkgsums` from `archlinux-contrib` inside `packaging/arch/`.
3. Build: `makepkg -si` (from `packaging/arch`).

## Winget (`winget/Sangkan.Cantrik.yaml`)

1. After publishing the release asset, compute SHA-256 of `cantrik` (Linux portable in manifest today matches [release.yml](../.github/workflows/release.yml)).
2. Set `InstallerSha256` (64 hex chars, no spaces).
3. Validate on Windows: `winget validate packaging/winget/Sangkan.Cantrik.yaml` (job opsional CI: [`.github/workflows/winget-validate.yml`](../.github/workflows/winget-validate.yml)).

## nfpm `.deb`

See comments in [`nfpm.yaml`](nfpm.yaml). Requires a built `target/release/cantrik`.

## Nix

- **Dev shell:** [`nix/README.md`](nix/README.md).
- **Install from source inside shell:** `cargo install --locked --path crates/cantrik-cli` from repo root.

## Homebrew

See [`homebrew/cantrik.rb`](homebrew/cantrik.rb).

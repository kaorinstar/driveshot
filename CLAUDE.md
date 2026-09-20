# CLAUDE.md

Guidance for AI coding assistants working in this repository.

## What this application is

Driveshot captures part of the screen, uploads it to a cloud drive the user already owns, shares
the link, and deletes the file once the retention runs out.

**The point is that the file stops existing.** A share link is public to whoever holds it, and
the retention is what makes that acceptable. Anything that weakens deletion — a file uploaded
without a record of it, a retention that silently becomes "forever", a failure swallowed rather
than retried — is a bug in the feature that matters most, not a rough edge.

**It is early.** Capture, upload, sharing and deletion are all unwritten. What exists is the
skeleton and the retention logic. Do not describe the application as doing something it does not
do yet, in a README, a commit message or a reply.

## Language policy

- **All code, comments, identifiers, commit messages, and documentation are written in English.**
  This project is to be published.
- A file whose name ends `.ja.md` is the Japanese translation of the file beside it:
  `README.ja.md`, `SECURITY.ja.md`, `version.ja.md` and `docs/architecture.ja.md`. When you change
  the English file, update its translation in the same commit so the two stay in sync.
- **User-facing strings are never written where they are drawn.** Each one has a name in
  `src/strings.ts`. The interface is in English alone today; it is written this way from the start
  because adding a second language then costs one more table, while pulling a hundred strings back
  out of the markup later costs an afternoon and misses some.

## Build and test

On Windows or macOS, everything:

```
npm install
npm run build
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

On Linux, the same commands work once the WebKitGTK development packages are installed, which is
how the application crate was verified when this repository was set up:

```
sudo apt-get install libwebkit2gtk-4.1-dev libsoup-3.0-dev libgtk-3-dev librsvg2-dev patchelf
```

A `cargo build` that fails with a message about `webkit2gtk`, `glib` or `soup` means they are
missing. Install them rather than reporting the build as broken. **What that build proves is that
the Rust compiles**, and no more: Linux is not a platform Driveshot is released for, the window
is drawn by a different engine there, and `release.yml` packages nothing for it. Without those
packages, this still runs and covers the logic that matters most:

```
cargo fmt --all -- --check
cargo clippy -p driveshot-core --all-targets -- -D warnings
cargo test -p driveshot-core
npm run build
```

Warnings are treated as errors. Do not call a task finished while one remains.

Dependency licences and advisories:

```
cargo install cargo-deny
cargo deny check
```

## Verification

A build proves that the code compiles, not that the application works. Capture, a hotkey, an
upload and a window's appearance cannot be confirmed from here at all.

After changing code, state clearly what a human needs to check by running the application, and on
which platform. "It builds" is not the same as "it works".

## Asking the user to do something

The environment an assistant works in cannot run this application. Anything that needs a real
machine is handed to the user, so writing that hand-off is part of the work rather than an
afterthought.

Give them this, in this order, and nothing else:

1. **What to do**, numbered in the order it is performed.
2. **The exact URL** for anything on GitHub. Never "from the Actions tab": paste the link.
3. **Commands that can be pasted as they are**, with the branch name and the paths already filled
   in. Not a template with a placeholder left in it.
4. **What to look at**, phrased so the answer is yes or no.
5. **What to send back** when the answer is no.

Leave out the design, the reasoning, and anything already reported. If the reasoning matters it
belongs in the pull request, not in the instruction.

### Handing over a build to test

**From GitHub, when there is no toolchain on the machine**

1. Open https://github.com/kaorinstar/driveshot/actions/workflows/release.yml
2. **Run workflow** → Branch: the branch to test → **Run workflow**. There are no other inputs.
3. When the run finishes, open it and download `driveshot-windows` or `driveshot-macos` from the
   Artifacts section at the bottom of the page.
4. Unzip it. It holds the installer for that platform.

A manual run publishes nothing. It stamps the version already in `src-tauri/Cargo.toml` and
creates no release; only pushing a `v*` tag does that.

**On a machine with Rust and Node**

```
git fetch origin <branch>
git checkout <branch>
npm install
npm run tauri build
```

The installer is under `src-tauri/target/release/bundle/`.

**`build.yml` is not a route to a build.** It compiles and tests and packages nothing, so there is
no artifact on it to download. Do not send anyone to a `build.yml` run for a file.

## Design rules

Full details are in `docs/architecture.md`. The rules that matter most:

1. **A calculation goes in `crates/driveshot-core` first.** Anything writable without a screen, a
   file or a network belongs there, where it compiles and is tested on every platform including
   this one. What stays in `src-tauri` is what genuinely cannot.
2. **Calls go downwards only**: the settings window calls the application through `invoke`, the
   application calls the core crate through plain function calls. The core crate knows nothing
   about Tauri, and the window knows nothing about how an upload happens.
3. **The version number lives in `src-tauri/Cargo.toml` under `[package]`, and nowhere else.**
   `tauri.conf.json` deliberately carries none, so the bundler reads that one. Do not add a
   second place to write it.
4. **Driveshot deletes only what its own record names.** A file the index does not hold is not
   Driveshot's to delete, whatever it looks like and wherever it sits.

## Conventions

- Line endings are LF, except `.bat`, `.cmd` and `.ps1`, which use CRLF. This is enforced by
  `.gitattributes`.
- Branch names are `<type>/<issue number>-<short description>`, for example `feat/3-region-capture`,
  `fix/18-retention-boundary`, `ci/7-split-workflow`, `docs/2-architecture`. The types are `feat`,
  `fix`, `docs`, `ci` and `chore`. Drop the issue number when the work has no issue.
  A Claude Code session is assigned a `claude/...` branch by default. **That name is not part of
  this convention** — it describes the session rather than the change. Point it out and move the
  work to a branch that follows the convention before pushing.
- `Cargo.lock` and `package-lock.json` are committed. This is an application, not a library: a
  build that reproduces is worth more than a dependency tree that drifts. Never edit either by
  hand; run `cargo`/`npm` and commit what it writes.
- Every GitHub Action is pinned to a full commit SHA with its version in a comment beside it.
  Keep it that way when adding one; `git ls-remote https://github.com/<owner>/<repo> refs/tags/<tag>`
  gives the SHA.

## Working while other sessions are open

Several sessions may be writing on this repository at the same time, so `main` moves while a
branch is being written. `.claude/hooks/check-main.sh` reports when that has happened and refuses
a push while `main` is unmerged; `/sync-main` does the merge. Merge, never rebase: these branches
are not private.

## Things that look redundant but are not

### `publish = false` in both `Cargo.toml` files

Neither crate goes to crates.io — the application is distributed as an installer. Without it,
`cargo deny check bans` reads `driveshot-core = { path = ... }` as a wildcard dependency on a
publishable crate and fails.

### `unmaintained = "workspace"` in `deny.toml`

Tauri reaches the unmaintained `unic-*` crates through `urlpattern`. There is no newer version to
move to and nothing here can change it, so the unmaintained check covers what this workspace
depends on directly. Known vulnerabilities are still refused anywhere in the tree, however deep.
Do not widen that back to `all` without a way to act on the result.

### The release profile build in `build.yml`

`panic = "abort"`, link-time optimisation and `strip` are release-only settings. A debug build and
a test run prove nothing about them, which is why the job builds `--release` as well without
packaging anything.

### `NonZeroU32` in `Retention::Days`

Zero days is not a retention period; it is an instruction to delete what was just uploaded. The
type refuses it, so a cleared input field or a settings file with `0` in it cannot cause that, and
no caller has to remember to check.

## Not yet implemented

In the order planned, and subject to the two open decisions in `docs/architecture.md`:

1. Tray icon and a global hotkey.
2. Region capture, saved locally, with no upload.
3. One cloud drive end to end: OAuth, upload, share link on the clipboard.
4. The record index on disk, and deletion when a retention runs out.
5. The remaining two cloud drives.
6. Settings that persist.

**Two decisions are open and are not an assistant's to settle by implementing them**: how many
cloud drives to support first, and whether deletion has to run when the user's machine is off.
Both are written out in `docs/architecture.md` with what each choice costs. If a task requires
one of them, say so and ask.

## Known limitations

- Nothing is code-signed, on either platform. SmartScreen warns on Windows; macOS refuses to open
  the application until it is allowed through System Settings.
- Retention only runs while Driveshot does. This is one of the open decisions above.
- macOS screen capture will need the "Screen Recording" permission. There is no way around it and
  Driveshot will not try to find one.
- The icon is drawn by `tools/make-icon.py` rather than by a designer.

## Continuous integration

- `.github/workflows/build.yml` — verification. Three jobs: `core` (Linux: formatting, clippy and
  tests for `driveshot-core`, plus the settings window), `app` (Windows and macOS: the whole
  workspace, including a release build), and `deny` (Linux: licences and advisories). It packages
  nothing and runs with `contents: read`.
- `.github/workflows/release.yml` — distribution. A `v*` tag publishes a release with a Windows
  installer and a macOS disk image; a manual run produces the same two as an artifact and
  publishes nothing. Only this workflow gets `contents: write`.
- `.github/workflows/report-build-status.yml` — called by both, for pushes only. Opens or
  comments on an issue labelled `ci-failure`, and comments again on the next success without
  closing it.

`build.yml` and `release.yml` carry the same build steps on purpose, so each can be read straight
through. **Change them together.** The one deliberate difference is the version: a release build
writes the tag into `src-tauri/Cargo.toml` first.

CodeQL is not set up: code scanning is free on public repositories only, so that workflow arrives
when this repository is published.

## Releasing

See "Releasing" in `README.md` for the full procedure. Two things are easy to get wrong:

1. **The `## Unreleased` heading is renamed to the version number**, in `version.md` and
   `version.ja.md`, in the pull request that prepares the release. Both files also mention
   `## Unreleased` in the paragraph that explains them, so rename the heading itself rather than
   the first match in the file. No tag matches `Unreleased`, so forgetting this fails the workflow
   instead of publishing an empty release.
2. **The same pull request sets the `version` under `[package]` in `src-tauri/Cargo.toml`** to the
   number being released. Nothing fails when this one is missed, which is the difficulty with it:
   the release itself takes its number from the tag and is correct either way, and only local
   builds and manual runs carry the stale value.

### An assistant cannot push the tag

The credentials a Claude Code session is given push branches, not tags. **Do not hand anyone
`git tag` and `git push` commands to run on their own machine.** Everything up to the tag is an
assistant's to do: the changelog heading, the version in `src-tauri/Cargo.toml`, the pull request
and its merge. Then hand over exactly this:

1. Open https://github.com/kaorinstar/driveshot/releases/new
2. **Choose a tag** → type the version, for example `v0.1.0` → pick
   **Create new tag: v0.1.0 on publish**
3. Check **Target** is `main`
4. Leave the title and the description empty. `release.yml` fills them from `version.md`
5. **Publish release**

Then watch it at
https://github.com/kaorinstar/driveshot/actions/workflows/release.yml and check the result at
https://github.com/kaorinstar/driveshot/releases.

## History

The repository was set up in a Claude Code session running on Linux. Everything in it was built
and tested there, the application crate included, but a Linux build only proves the code
compiles: the window has never been opened, and nothing here has run on Windows or macOS. The
first `release.yml` run on a real machine is where that changes, and it is the first thing worth
doing.

## Choosing a model for subagents

**When calling the Agent tool, name the `model` explicitly. Default to a light model
(`sonnet` or `haiku`).**

| Model | Work to send there |
|---|---|
| `haiku` / `sonnet` | Locating files, grep-style enumeration, mechanical checks, routine work |
| `opus` | Cross-cutting contradiction hunts, design judgement, anything hard to undo |

- **Explore defaults to a light model** — its job is to find where things are.
- **Use Opus when you judge it necessary.** This is not an instruction to economise; it is an
  instruction not to spend Opus on work a light model already handles.
- **Do not take a light model's answer on trust.** If the output is shallow or looks like it
  missed something, send it again on Opus rather than building on it.

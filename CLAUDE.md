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
- **User-facing strings are never written where they are drawn.** The settings window's are in
  `src/strings.ts`; what the window cannot reach - the tray menu, its tooltip, the sentences about
  a hotkey that would not register - is in `src-tauri/src/strings.rs`. The interface is in English
  alone today; it is written this way from the start because adding a second language then costs
  one more table per side, while pulling a hundred strings back out of the markup later costs an
  afternoon and misses some.

## Build and test

On Windows or macOS, everything:

```
npm install
npm run build
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

On Linux, the same commands work once the system libraries are installed. The list has grown as
features arrived — the web view first, then the tray, then screen capture — and this is all of it:

```
sudo apt-get install libwebkit2gtk-4.1-dev libsoup-3.0-dev libgtk-3-dev librsvg2-dev \
  patchelf libayatana-appindicator3-dev libpipewire-0.3-dev libgbm-dev libdrm-dev \
  libegl1-mesa-dev libwayland-dev libclang-dev clang
```

A `cargo build` that fails naming one of those libraries, or a link that cannot find `-lgbm`,
means the list is incomplete on that machine. Install them rather than reporting the build as
broken. Most of them exist only for Linux: the tray comes from the operating system on Windows and
macOS, and `xcap` reaches for pipewire, gbm and wayland only in its Linux backend. **What that build proves is that
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

The installer is under `target/release/bundle/` — the workspace's build directory, at the
root. `src-tauri/target/` does not exist and never will (#14).

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

### Starting the application puts nothing on the screen

`tauri.conf.json` gives the window `"visible": false`, and `src-tauri/src/main.rs` prevents its
close and hides it instead. Driveshot is the tray icon; the window is one way of looking at it.
Quitting is the tray menu's last entry, and nothing else exits the application.

### Both the `Info.plist` and the activation policy ask macOS for the same thing

`src-tauri/Info.plist` sets `LSUIElement` so the bundle has no Dock icon, and `main.rs` asks for
`ActivationPolicy::Accessory` at runtime. The plist covers the packaged application; the runtime
call covers `tauri dev`, which never reads it. Neither has been tested — nobody has run the macOS
build.

### Nothing asks a platform for a display's scale factor

`driveshot_core::pixels_for` works out the scale from the size of the surface the user drew on and
the size of the image that was captured. That looks like the long way round, and it is there
because the short way is wrong: `xcap::Monitor::width` returns a **logical** width on Linux (it
divides by the scale factor) and a **physical** one on Windows (`dmPelsWidth`). Code written
against either is broken on the other, and both look right at 100% where the two are equal.

For the same reason `capture_region` is not used, and monitors are matched to what `xcap` captures
by `Monitor::from_point` with a physical point inside them — that one means the same thing
everywhere.

### `macos-private-api`, and what it costs

The capture overlay is a transparent window. On macOS that needs Tauri's `macos-private-api`
feature and `"macOSPrivateApi": true` in `tauri.conf.json`; without them `.transparent()` does not
exist on that platform and the build fails there while compiling everywhere else (#17).

**An application built with it cannot go in the App Store.** Driveshot is distributed as a disk
image from its releases page, so nothing planned is lost, but do not turn the flag off to "avoid
private API" without replacing the overlay: the alternative is showing a captured image in an
opaque window instead of dimming a transparent one, which is a different design, not a smaller
one.

### `NonZeroU32` in `Retention::Days`

Zero days is not a retention period; it is an instruction to delete what was just uploaded. The
type refuses it, so a cleared input field or a settings file with `0` in it cannot cause that, and
no caller has to remember to check.

## Not yet implemented

In the order planned, and subject to the two open decisions in `docs/architecture.md`:

1. ~~Tray icon and a global hotkey (#4)~~ — done.
2. ~~Region capture, saved locally, with no upload (#5)~~ — done.
3. One cloud drive end to end: OAuth, upload, share link on the clipboard (#6).
4. The record index on disk, and deletion when a retention runs out (#7).
5. The remaining two cloud drives (#8).
6. Settings that persist (#9).

**Two decisions are open and are not an assistant's to settle by implementing them**: how many
cloud drives to support first (#2), and whether deletion has to run when the user's machine is
off (#3). Both are written out in `docs/architecture.md` with what each choice costs, and each
issue says what decides it. If a task requires one of them, say so and ask.

Every issue is open at the time of writing, and each names what it depends on. Read the issue
before starting the work: it carries the constraints that are not in the code, such as why the
OAuth client secret cannot be kept secret (#6) and why a failed deletion must leave its record in
place (#7).

## Known limitations

- Nothing is code-signed, on either platform. SmartScreen warns on Windows; macOS refuses to open
  the application until it is allowed through System Settings (#12).
- Retention only runs while Driveshot does. This is one of the open decisions above (#3).
- macOS screen capture will need the "Screen Recording" permission (#5). There is no way around it
  and Driveshot will not try to find one.
- The icon is drawn by `tools/make-icon.py` rather than by a designer.
- The name has not been checked against a trademark database, only searched for on GitHub and in
  the application stores (#11).

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
when this repository is published (#10).

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
and tested there, the application crate included. `build.yml` then built and tested it on Windows
and macOS as well, and passed on its first run (#1).

`release.yml` has since produced both installers, and **the Windows one has been installed and
run by hand**, twice. The first time proved the path from source to a running application: the
installer completes, the window opens, and it shows the three drives and the retention selector.
The second checked the tray and the hotkey from #4 — the icon and its three entries, a left click
opening the window, the window hiding rather than closing, Ctrl+Shift+D working while another
application had focus, and Quit leaving nothing running. All of it behaved.

**macOS has not been run.** Its disk image is built by the same workflow and nothing suggests it
is broken, but nobody has opened it. Treat anything about how the application behaves on macOS as
unverified until someone does, and say so rather than implying otherwise.

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

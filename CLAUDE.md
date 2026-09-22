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
  This repository is public (#10), so anything written here is written for a stranger to read.
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
2. **Run workflow** → Branch: the branch to test → **Platforms**: `both`, or the one platform the
   build is for → **Run workflow**.
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
- **A change starts from an issue.** Write one before the code. It carries what never reaches the
  diff — why a decision went the way it did, what was ruled out, and what a human has to check on a
  real machine — and it is what the branch name and the pull request point back to. Finding no
  issue for a task is not permission to begin without one; it means the issue is the first thing to
  write.
- Branch names are `<type>/<issue number>-<short description>`, for example `feat/3-region-capture`,
  `fix/18-retention-boundary`, `ci/7-split-workflow`, `docs/2-architecture`. The types are `feat`,
  `fix`, `docs`, `ci` and `chore`. The number is dropped only where there is genuinely nothing for
  an issue to say — a typo, a comment, a file moved. A branch without one is otherwise a sign that
  the issue was skipped, not a second ordinary way of naming a branch.
  A Claude Code session is assigned a `claude/...` branch by default. **That name is not part of
  this convention** — it describes the session rather than the change. Point it out and move the
  work to a branch that follows the convention before pushing.
- `Cargo.lock` and `package-lock.json` are committed. This is an application, not a library: a
  build that reproduces is worth more than a dependency tree that drifts. Never edit either by
  hand; run `cargo`/`npm` and commit what it writes.
- Every GitHub Action is pinned to a full commit SHA with its version in a comment beside it.
  Keep it that way when adding one. `git ls-remote https://github.com/<owner>/<repo> refs/tags/<tag>`
  gives the SHA of an annotated tag's *tag object*, which is not what a workflow can pin; add
  `^{}` to the ref, or read the `refs/tags/<tag>^{}` line, to get the commit.
- **A commit message ends with `Co-Authored-By:` and nothing else.** A link to the assistant
  session that produced the change belongs in the pull request, which is where the reasoning for a
  change is written anyway; in a commit it is a second copy of that link, permanent, and openable
  by one person. Four commits made before this rule still carry one. They are on `main` and stay
  there: this is not worth rewriting shared history for.

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

### `shadow(false)` on the capture overlay

It reads as a cosmetic setting on a window nobody looks at the edges of. It is not. Tauri's
`shadow` defaults to `true`, and on Windows an undecorated window with a shadow keeps its resize
frame: `tao` answers `WM_NCCALCSIZE` by pulling the page inside in by that frame's width — eight
physical pixels at 100%, ten at 125%, twelve at 150% — and Tauri's own documentation adds "a 1px
white border". On an overlay meant to cover one monitor exactly, that is a strip of undimmed
screen down each side, a white line around the lot, and a selection measured against a surface
wider than the real one (#22).

### An overlay is placed after it is built, not by its builder

`WebviewWindowBuilder::position` takes **points**. `Monitor::position` returns **physical pixels**.
Passing one to the other is right only on a monitor whose origin is `(0, 0)`, and the builder's own
conversion uses whatever scale factor the window is created under rather than the one belonging to
the monitor it is being sent to. So the builder is given a reasonable starting point and
`set_position`/`set_size` then place the window in physical pixels, where nothing is converted.

### The overlays are hidden rather than closed, and built before anyone asks for one

A capture does not create the overlay windows and does not destroy them. They are built at
startup, one per monitor, hidden; a capture shows them and hides them again. That reads as a leak
— a window nobody can see, held for the life of the application, on a tool that idles in the tray
— and it is deliberate. Building a web view and loading a page into it took about a second, and
that second was the entire delay between pressing the key and the screen dimming (#23).

What it costs is real and is the reason this is written down rather than assumed: a WebView2
process per monitor for as long as Driveshot runs, a set of windows that has to be reconciled
against `available_monitors` on every capture because a monitor can be plugged in or unplugged in
between, and a page that still holds the last capture's selection when the next one starts. The
last of those is why a capture begins by emitting `capture-begin`: each overlay puts itself back
to how it began and answers `overlay_ready`, and only then is it shown. That handshake is also
what keeps a blank web view off the screen (#20) — it is now a round trip rather than a page load.

### `macos-private-api`, and what it costs

The capture overlay is a transparent window. On macOS that needs Tauri's `macos-private-api`
feature and `"macOSPrivateApi": true` in `tauri.conf.json`; without them `.transparent()` does not
exist on that platform and the build fails there while compiling everywhere else (#17).

**An application built with it cannot go in the App Store.** Driveshot is distributed as a disk
image from its releases page, so nothing planned is lost, but do not turn the flag off to "avoid
private API" without replacing the overlay: the alternative is showing a captured image in an
opaque window instead of dimming a transparent one, which is a different design, not a smaller
one. #18 has that design written out, with what it would cost.

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

- `.github/workflows/build.yml` — verification. Four jobs: `plan` (Linux: which platforms the
  application is built on), `core` (Linux: formatting, clippy and tests for `driveshot-core`, plus
  the settings window), `app` (the whole workspace on the platforms `plan` names, including a
  release build), and `deny` (Linux: licences and advisories). It packages nothing and runs with
  `contents: read`.
- `.github/workflows/release.yml` — distribution. A `v*` tag publishes a release with a Windows
  installer and a macOS disk image; a manual run produces the same as an artifact and publishes
  nothing. Only this workflow gets `contents: write`.
- `.github/workflows/codeql.yml` — static analysis of this project's own code, which the
  dependency check covers none of. One job per language, `javascript-typescript` and `rust`, with
  nothing built for either: both are extracted from source. **Rust support is in public preview**,
  so a quiet result there means nothing was reported rather than that there is nothing to report.
- `.github/workflows/report-build-status.yml` — called by `build.yml` and `release.yml`, for
  pushes only. Opens or comments on an issue labelled `ci-failure`, and comments again on the next
  success without closing it.

`build.yml` and `release.yml` carry the same build steps on purpose, so each can be read straight
through. **Change them together.** The one deliberate difference is the version: a release build
writes the tag into `src-tauri/Cargo.toml` first.

### Which platforms a run builds on

`APP_PLATFORMS` at the top of `build.yml` holds one word — `windows`, `macos` or `both` — and it
is the only place a push or a pull request takes the answer from. It is `windows` today: the work
is aimed at Windows first, then at macOS, and only then at both.

**The reason it says `windows` has gone, and the value has not caught up yet (#30).** #19 chose
it to save billed minutes: this repository was private, GitHub charged a macOS minute at ten times
a Linux one, and roughly 50 of one run's 80 billed minutes were the macOS job alone. Standard
runners are free on a public repository, and this one is public as of #10, so there is nothing left
to save.

Waiting was never the argument either. Measured on
[the run on `main` that built both](https://github.com/kaorinstar/driveshot/actions/runs/35595214662),
the macOS job took 4m20s and the Windows job 12m16s, and they run at the same time; with the Rust
cache warm the Windows job took 5m05s. Windows decides how long a run takes, so dropping macOS
leaves a run about as long as it was.

What it costs is unchanged, and is now the only thing the value is trading against:

- **While it says `windows`, nothing checks that the code still compiles on macOS.** The first run
  after it is widened again finds everything that broke in between, at once.
- Run `build.yml` by hand on the other platform now and then rather than waiting for that.
  **Run workflow** → **Platforms** → `macos` overrides `APP_PLATFORMS` for that run alone, without
  a commit.
- **Set it back to `both` before a release.** A tag ignores it — `release.yml` always packages both
  platforms, because a release carrying one of them is not a release — but a release is a poor
  place to discover that the other platform stopped compiling three weeks ago.

`release.yml` takes the same three words as a **Platforms** input on a manual run, defaulting to
`both`. A tag ignores the input entirely.

Every action is pinned to a full commit SHA. `.github/dependabot.yml` watches Cargo, npm and the
actions weekly so that pinning does not mean going stale, and groups the two halves of
`github/codeql-action`: a workflow whose `init` and `analyze` come from different versions fails
with a configuration error after uploading nothing.

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

The third checked capture (#5), and **the saved image matched the selection at 100%, 125% and
150% display scaling**. That is the design in "Nothing asks a platform for a display's scale
factor" holding up on the platform it was written against. The same run found two faults in how
the overlay appears: #20, where it flashed white before it dimmed, and #22, where it sat about ten
pixels right of the monitor's left edge.

The fourth checked both fixes, and **both hold: no white frame, and the dimming reaches the edges
of the screen**. That run also tried **more than one monitor for the first time, and capture
worked there** — so the physical placement in `open_overlay` and the matching by
`Monitor::from_point` are right on the desktop they were written for, rather than only on paper.

What it found instead is that **the overlay now takes about a second to appear** (#23). The white
frame was that same second, spent with a window on the screen rather than without one; waiting for
the page removed the flash and left the wait visible.

More runs by hand have happened since, all on Windows. One, on a build from before #29, found
that **the saved image carried the overlay's dimming over every colour in it** (#34). A capture
closed the overlays and then waited 120 ms on the main thread for the screen to clear;
`tauri-runtime-wry` sends every close through the event loop, so that wait was what stopped the
screen clearing.

#29 hides the overlays instead of closing them, for the speed in #23, and **its own run confirmed
the speed**. Hiding is carried out where it is asked for rather than queued, so it removed the
cause of #34 as well, without setting out to. **A run on a build from `main` with #29 in it
confirmed that: the saved image has the screen's own colours.** Nothing was written for #34 in the
end; the pull request that had been opened for it was closed unmerged, because it was built on the
overlays being closed.

A later run, on a build from `main` with #24 in it, checked the new icon. **The tray icon reads
as a cloud rather than the grey smudge the old one left**, and the taskbar and the installer carry
it correctly. That is what drawing each size at its own resolution buys: `npx tauri icon`
resampled one large image down to 16 pixels, which is what the tray asks for at 100% scaling, and
a mark that detailed does not survive it.

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

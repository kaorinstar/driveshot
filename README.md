# driveshot

[日本語版はこちら / Japanese version](README.ja.md)

Capture part of the screen, keep it on a cloud drive you already own, share the link, and have the
file delete itself when its retention runs out.

**Driveshot is in early development and has not been released.** Nothing is captured, uploaded,
shared or deleted yet. What exists is the skeleton described under [Status](#status). This README
describes what is being built, and says plainly which parts are not there.

## Why

Sharing a screenshot usually means uploading it to somebody else's service, where it stays until
that service decides otherwise. Driveshot puts the file on a drive the user already pays for and
controls — Google Drive, OneDrive or Dropbox — and then removes it again on a schedule the user
sets. The link is convenient for as long as it is needed, and stops existing afterwards.

## What it will do

1. **Capture** a region of the screen, from a hotkey.
2. **Upload** it to a cloud drive, over that provider's own OAuth sign-in. Driveshot never sees a
   password.
3. **Share** it: the uploaded file is set to "anyone with the link can view" and the URL is put on
   the clipboard.
4. **Delete** it once the retention has run out — a day, a week, a month, or never, chosen by the
   user.

## Status

| Part | State |
|---|---|
| Repository, build, tests, CI, packaging | Working |
| Installing and starting it (Windows) | Verified by hand |
| Installing and starting it (macOS) | Built, not yet run by anyone |
| Retention and upload-record logic (`driveshot-core`) | Written and tested |
| Settings window | A skeleton: it lists the drives and shows when a shot would expire |
| Tray icon and global hotkey | Verified by hand on Windows |
| Screen capture | Not started |
| Cloud upload and OAuth | Not started |
| Share links | Not started |
| Automatic deletion | Not started |

Two design decisions are still open: how many cloud drives to support first
([#2](https://github.com/kaorinstar/driveshot/issues/2)), and whether deletion has to run when the
user's machine is off ([#3](https://github.com/kaorinstar/driveshot/issues/3)). Both are written
out in [docs/architecture.md](docs/architecture.md), including what each choice costs.

## Requirements

- **Windows 10 or later.** The window is drawn by WebView2, which is part of Windows 11 and is
  installed on virtually every Windows 10 machine through Windows Update.
- **macOS 10.15 or later.** The window is drawn by WKWebView, which is part of macOS; nothing
  extra is installed. Screen capture will additionally need the "Screen Recording" permission,
  which macOS asks for the first time it is used.

## Build from source

You need [Rust](https://www.rust-lang.org/tools/install) (stable) and
[Node.js](https://nodejs.org/) 22 or later.

```
npm install
npm run tauri build
```

The installer is written under `target/release/bundle/`. That is the workspace's build
directory, at the root — `src-tauri` is a member of the workspace, so it has no `target/` of
its own.

To run it while working on it:

```
npm run tauri dev
```

**Linux is not a platform Driveshot is released for**, and no Linux package is built. The source
does compile there, which is worth knowing because it is where most of the development happens.
Tauri draws its window with the operating system's own web view, so building it needs the
WebKitGTK development packages; on Ubuntu or Debian:

```
sudo apt-get install libwebkit2gtk-4.1-dev libsoup-3.0-dev libgtk-3-dev librsvg2-dev patchelf
```

Without them, only the application crate is out of reach, and everything that matters most still
runs:

```
cargo fmt --all -- --check
cargo clippy -p driveshot-core --all-targets -- -D warnings
cargo test -p driveshot-core
npm run build
```

Warnings are treated as errors everywhere. Do not call a task finished while one remains.

## Repository layout

```
crates/driveshot-core/   Logic with no UI and no OS dependency. Tested on every platform.
src-tauri/               The application: window, commands, and later the capture and uploads.
src/                     The settings window: HTML, CSS, TypeScript.
docs/                    Architecture, in English and Japanese.
tools/make-icon.py       Draws the application icon. See the comment at the top of the file.
.github/workflows/       Verification (build.yml) and distribution (release.yml).
```

`crates/driveshot-core` is where a new calculation goes first.
[docs/architecture.md](docs/architecture.md) says why.

## Continuous integration

Two workflows, split so that a verification run never needs write access to the repository.

- **`.github/workflows/build.yml`** runs on every push to `main` and every pull request. It
  checks the formatting, lints and tests `driveshot-core` on Linux, builds and tests the whole
  workspace on Windows and macOS, builds the settings window, and checks every dependency for
  known vulnerabilities and for a licence this project can ship. It packages nothing and runs
  with `contents: read`.
- **`.github/workflows/release.yml`** builds, tests and packages. Pushing a tag such as `v0.1.0`
  publishes a release with two files attached: a Windows installer and a macOS disk image.
  Starting it by hand produces the same two as a build artifact and creates no release, which is
  the only way to try a change to the packaging before a tag exists. Only this workflow gets
  `contents: write`.

A third file, `.github/workflows/report-build-status.yml`, is called by both once their jobs
finish, and only for pushes. On a failure it opens an issue labelled `ci-failure`, or comments on
the one already open rather than opening a second; on the next success it comments on that issue
too. It never closes it: a green build shows the symptom is gone, not that the cause was
understood.

Every action is pinned to a full commit SHA, with its version in a comment beside it, because a
tag is a pointer its owner can move. `.github/dependabot.yml` watches Cargo, npm and the actions
weekly, so pinning does not mean going stale.

Dependency licences are checked by [cargo-deny](https://github.com/EmbarkStudios/cargo-deny)
against `deny.toml`, which lists every licence allowed and says why. Run the same check locally
with `cargo install cargo-deny && cargo deny check`.

**CodeQL is not set up yet.** Code scanning is free on public repositories only, so that workflow
arrives when this repository is published
([#10](https://github.com/kaorinstar/driveshot/issues/10)).

## Releasing

Tags are `vMAJOR.MINOR.PATCH`, as semantic versioning describes, for example `v0.1.0`. Below
`1.0.0` the minor number covers additions and changes, and the patch number covers fixes alone.
No leading zeros.

`version.md` is the changelog, newest version at the top, one section per version.
`version.ja.md` is its Japanese translation and is updated in the same commit. `release.yml`
reads the section whose heading matches the tag and uses it as the release notes, so the entry
has to be committed **before** the tag is pushed. A malformed tag, a missing section or an empty
one fails the workflow before anything is built.

Every change a user would notice adds its entry to `## Unreleased` in the same pull request that
makes the change, in both files. Preparing a release is then renaming that heading to the version
number, and setting the `version` under `[package]` in `src-tauri/Cargo.toml` to the same number,
rather than reconstructing the list from the commit log afterwards.

Publishing itself is done from
[the releases page](https://github.com/kaorinstar/driveshot/releases/new): choose a new tag such
as `v0.1.0` on `main`, leave the title and description empty, and publish. The tag starts
`release.yml`, which fills both from `version.md` and attaches the packages.

## Known limitations

- **Nothing is code-signed.** The Windows installer is unsigned, so SmartScreen warns the first
  time it runs. The macOS application is neither signed nor notarized, so macOS refuses to open it
  until it is allowed through System Settings → Privacy & Security. Both are fixed by buying a
  certificate, which has not been done
  ([#12](https://github.com/kaorinstar/driveshot/issues/12)).
- **Retention only runs while Driveshot does.** A machine that is off past a file's retention
  leaves that file in place until the next start. See
  [docs/architecture.md](docs/architecture.md).
- **The icon is drawn in code** by `tools/make-icon.py`, not by a designer. It is the icon the
  application ships with until a better one replaces it.

## Roadmap

In the order it is planned, and subject to the two open decisions above.

1. ~~Tray icon and a global hotkey~~ ([#4](https://github.com/kaorinstar/driveshot/issues/4)) — done.
2. Region capture, saved locally, with no upload ([#5](https://github.com/kaorinstar/driveshot/issues/5)).
3. One cloud drive end to end: OAuth sign-in, upload, share link on the clipboard ([#6](https://github.com/kaorinstar/driveshot/issues/6)).
4. The record index on disk, and deletion when a retention runs out ([#7](https://github.com/kaorinstar/driveshot/issues/7)).
5. The remaining two cloud drives ([#8](https://github.com/kaorinstar/driveshot/issues/8)).
6. Settings that persist: retention, hotkey, which drive ([#9](https://github.com/kaorinstar/driveshot/issues/9)).

## Contributing

An issue first, please, before a large change. The project is small and the design decisions it
has not made yet are written down in `docs/architecture.md`; a pull request that settles one of
them by implementing it is a harder conversation than one that starts with the decision.

Code, comments, identifiers, commit messages and documentation are written in English. A file
whose name ends `.ja.md` is the Japanese translation of the file beside it and is updated in the
same commit.

## Security

Report a vulnerability privately, through
[GitHub's form](https://github.com/kaorinstar/driveshot/security/advisories/new) rather than as
an issue. [SECURITY.md](SECURITY.md) says what the application touches, what it is being built to
touch, and what is already known.

## License

[MIT](LICENSE).

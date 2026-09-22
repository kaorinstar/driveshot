# Architecture

[日本語版はこちら / Japanese version](architecture.ja.md)

## The shape of the repository

Three layers, and the boundary between the first two is the rule everything else follows.

```
src/                      The settings window: HTML, CSS and TypeScript. Draws answers.
  ↓  invoke()
src-tauri/                The application: tray, hotkey, capture, cloud calls, files.
  ↓  plain function calls
crates/driveshot-core/    Decisions. No window, no network, no operating system API.
```

Calls only ever go downwards. The core crate knows nothing about Tauri, and the window knows
nothing about how an upload happens.

### Why the core crate exists

Everything in `crates/driveshot-core` compiles and is tested on any platform, including the Linux
machines this project is often worked on. That is not a convenience; it is what makes the risky
part of Driveshot testable at all. Deciding that a file has outlived its retention is a decision
that deletes somebody's screenshot when it is wrong, and it is a decision that can be written as a
pure function over a timestamp. So it is, and there are tests over every boundary of it.

The rule to apply when adding something: if it can be written without a screen, a file or a
network, it belongs in the core crate. What is left in `src-tauri` is the part that genuinely
cannot.

### What is in the core crate today

| Type | What it decides |
|---|---|
| `Provider` | Which cloud drive a file went to, and the identifier that names it on disk |
| `Retention` | How long a file is kept, and the moment it falls due for deletion |
| `ShotRecord` | One upload: the file, the drive, the time, the share link, the terms |
| `ShotIndex` | Every upload not yet deleted, and which of them are due |

Two decisions in there are deliberate and should not be undone without a reason:

- **A retention of zero days cannot be expressed.** `Retention::Days` holds a `NonZeroU32`, so a
  cleared input field or a settings file with `0` in it cannot mean "delete what was just
  uploaded". The type refuses it rather than every caller having to remember to check.
- **A due date that cannot be represented leaves the file alone.** Adding a very large number of
  days to a timestamp can overflow the calendar. That case is treated as "never due", not as
  "already due", because the second would delete a file the user asked to keep.

### Where the version number lives

In `src-tauri/Cargo.toml`, under `[package]`, and nowhere else. The Tauri bundler reads it from
there because `src-tauri/tauri.conf.json` deliberately does not carry one, and
`.github/workflows/release.yml` rewrites that single line from the tag it was started by. A second
place to write the number would be a second place to forget it.

## What the application does not do yet

The four things Driveshot is for are all unbuilt:

1. **Capture** a region of the screen.
2. **Upload** it to a cloud drive over OAuth.
3. **Share** it by setting the file to "anyone with the link can view" and taking the URL.
4. **Delete** it once the retention has run out.

Each has an issue: capture is [#5](https://github.com/kaorinstar/driveshot/issues/5), the first
cloud drive end to end is [#6](https://github.com/kaorinstar/driveshot/issues/6), the record index
and deletion are [#7](https://github.com/kaorinstar/driveshot/issues/7), and the tray and hotkey
that come before all of them are [#4](https://github.com/kaorinstar/driveshot/issues/4).

The window today lists the three providers and shows when a shot uploaded now would be deleted.
Both answers come from the core crate, through two Tauri commands in `src-tauri/src/main.rs`. That
is a skeleton on purpose: it makes the whole path - window, application, core, tests, CI,
installer - real and provable before any of the hard parts are written on top of it.

## Decisions: one settled, one still open

Neither had to be made to keep building, and both change what is built, so they were written down
rather than settled by accident. One of them has now been made.

### Which provider comes first: Google Drive, alone ([#2](https://github.com/kaorinstar/driveshot/issues/2))

**Settled.** Google Drive is implemented first and on its own. OneDrive and Dropbox follow
afterwards, as #8.

Google Drive, OneDrive and Dropbox all differ in how they authenticate, how they upload, and how
they publish a link. `Provider` names all three from the start, because the identifier of a
provider is written into every stored record and a record written today has to remain readable
later. That part was never in question; how many of them actually work in the first release was.

What decided it is that Google Drive is the drive this project's author uses. A provider nobody
here has an account on is a provider that cannot be tested, and an upload that has never been run
against a real account is not an upload anyone should trust.

**What it costs, written down because it is a real cost rather than a theoretical one:** the
abstraction will be shaped around Google Drive, because there is nothing else pushing against it.
When OneDrive and Dropbox arrive, that shape is expected to be wrong in places, and reshaping it
is part of #8 rather than a sign that something went wrong here. The alternative was to build
three at once and ship nothing until the third one worked.

### Where deletion runs ([#3](https://github.com/kaorinstar/driveshot/issues/3))

**Still open.**

Retention is the feature that makes a public share link acceptable, so the question is how
reliably it can run.

- **In the application alone.** Driveshot deletes what is due whenever it is running. Nothing
  else has to exist, and there is no server to pay for or keep up. A machine that is off past a
  file's retention leaves that file in place until the next start.
- **With a server.** A small service holds the schedule and deletes on time whatever the user's
  machine is doing. Retention then means what it says, at the cost of a service that has to be
  run, and of that service holding a token to the user's own cloud drive - which is a much larger
  thing to be responsible for than a desktop application that holds only its own.

The second is not a small addition to the first. It changes what Driveshot is: from an
application that touches only the user's machine and the user's drive, to a service that holds a
credential to somebody else's storage. If it is chosen, `SECURITY.md` has to say so plainly
before the release that introduces it.

Until this is decided, the core crate is written so that either can be built on it: `ShotIndex`
answers "what is due at this moment" and deletes nothing itself. Whoever calls it - the
application on a timer, or something else entirely - is left open.

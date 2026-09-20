# Version history

The newest version is at the top. Each entry says what changed for someone using the application,
rather than which pull requests were merged.

A release takes its notes from the section whose heading matches its tag. Entries are written as
each change lands, under `## Unreleased`, and that heading is renamed to the version number when
the release is prepared. Nothing matches `Unreleased`, so a tag pushed while entries are still
sitting there fails the release workflow rather than publishing an empty release. See "Releasing"
in [README.md](README.md).

`version.ja.md` is the Japanese translation of this file and is updated in the same commit.

## Unreleased

- Nothing has been released yet. The first release will be `v0.1.0`.
- Driveshot now sits in the tray rather than opening a window. Starting it puts nothing on the
  screen: its icon appears beside the clock, and the menu on that icon holds **Capture**,
  **Settings** and **Quit Driveshot**. Closing the settings window hides it again; the application
  keeps running until you quit it from that menu.
- **Ctrl+Shift+D**, or **Cmd+Shift+D** on a Mac, is the capture key. It works wherever you are,
  without bringing the window up first. The combination cannot be changed yet.
- If another application already holds that combination, Driveshot says so instead of leaving you
  with a key that quietly does nothing: it opens the settings window at startup and names what
  refused it.
- **Capture itself is not built yet.** Pressing the key, or choosing Capture from the menu, only
  brings the settings window up and notes the time. Uploading, sharing and deleting are not built
  either. The roadmap in [README.md](README.md) says in which order they arrive.

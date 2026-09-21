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
- **The key now takes a shot.** It dims every screen; drag a rectangle on one of them and that
  part of it is saved. Escape, a right click, or a click without a drag puts the overlay away and
  takes nothing.
- Shots are saved to a **Driveshot folder inside your pictures folder**, named after the moment
  they were taken — `driveshot-20260920-143052.png`. That is where they stay for now.
- A shot that could not be taken or could not be saved is not passed over in silence: the settings
  window opens and says what went wrong. On a Mac that includes being refused permission to record
  the screen, which macOS asks about the first time.
- The settings window shows where the last shot went, so you can tell it worked without going
  looking for the file.
- Pressing the key no longer turns the screen white for an instant before it dims.
- The dimming now reaches the edges of the screen. It used to stop about ten pixels short
  down each side, which also shifted the saved image a little away from what was selected.
- **Uploading, sharing and deleting are still not built.** A shot stays on your own machine. The
  roadmap in [README.md](README.md) says in which order the rest arrives.

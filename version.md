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
- The screen dims as soon as you press the key. It used to take about a second, because
  Driveshot built a window for each screen at that moment; those windows are now ready before
  you press anything.
- **The icon is new.** It is a cloud with an upward arrow inside it, held in a capture frame. It
  is also sharper: every size is now drawn at its own resolution rather than shrunk down from one
  large image, which is what left the tray icon looking soft. Below 40 pixels — the tray, and the
  small entries in the taskbar and the file list — the frame is dropped and the cloud is filled
  in, because three white strokes that close together cannot be told apart at that size.
- **Clicking the tray icon now takes a shot** instead of opening the settings window. It does
  the same as the capture key, so the shot you want is one click away rather than a menu away.
  The settings window is still on the menu that the right button opens.
- **The macOS application now starts.** It was killed the moment it was opened, because what
  Driveshot ships for macOS carried no signature at all. It is still not signed by a certificate
  Apple recognises, so the first time you open it macOS says it cannot verify it: allow it through
  System Settings → Privacy & Security, and it opens from then on.
- **Capture now works on macOS.** It used to fail with "Monitor not found" every time, on any Mac
  whose display is scaled up — which is every Mac with a Retina screen. Nothing was saved and
  macOS never asked to record the screen.
- **Shots taken on a Mac no longer carry the dimming.** The whole image used to come out darkened,
  because the screen was photographed before the shading had left it. Driveshot also no longer
  freezes for a moment when a shot is taken.
- **Uploading, sharing and deleting are still not built.** A shot stays on your own machine. The
  roadmap in [README.md](README.md) says in which order the rest arrives.

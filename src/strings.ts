// Every word the window shows is named here rather than written where it is drawn.
//
// The interface is in English alone today. It is written this way from the start because adding
// the second language then costs one more table and nothing else, while pulling a hundred strings
// back out of the markup later costs an afternoon and misses some.

export const strings = {
  subtitle: "Capture part of the screen, keep it on a drive you own, share the link.",

  destinationHeading: "Where shots are saved",
  destinationNote:
    "All three drives are listed from the start. None of them works yet; the one being built first is decided in the roadmap.",
  providerNotAvailable: "Not available yet",

  overlayHint: "Drag to select an area. Esc to cancel.",

  lastShotHeading: "The last shot",
  lastShotNone: "Nothing has been captured yet.",
  lastShotSaved: (path: string) => `Saved to ${path}`,
  lastShotUnavailable: "What happened to the last shot could not be read.",

  hotkeyHeading: "The capture key",
  hotkeyNote:
    "Driveshot sits in the tray and waits for this key. It works wherever you are, without bringing this window up first.",
  hotkeyHeld: (shortcut: string) => `${shortcut} is yours.`,
  hotkeyNeverPressed: "It has not been pressed yet.",
  hotkeyLastPressed: (when: string) => `Last pressed at ${when}.`,
  hotkeyUnavailable: "The capture key could not be read.",
  captureNotBuilt:
    "Capture itself is not built yet, so pressing the key only brings this window up.",

  retentionHeading: "How long a shot is kept",
  retentionNote:
    "A shot is deleted from the drive once its retention has run out. The retention is fixed when the shot is uploaded, so changing this later leaves shots already shared on their own terms.",
  retentionLabel: "Retention",
  retentionDays: (days: number) => `${days} days`,
  retentionForever: "Keep until I delete it",

  expiryForever: "A shot uploaded now would stay until you delete it yourself.",
  expiryAt: (when: string) => `A shot uploaded now would be deleted on ${when}.`,
  expiryFailed: "The retention could not be read.",
} as const;

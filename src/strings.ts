// Every word the window shows is named here rather than written where it is drawn.
//
// The interface is in English alone today. It is written this way from the start because adding
// the second language then costs one more table and nothing else, while pulling a hundred strings
// back out of the markup later costs an afternoon and misses some.

export const strings = {
  subtitle: "Capture part of the screen, keep it on a drive you own, share the link.",

  destinationHeading: "Where shots are saved",
  destinationNote:
    "All three drives are listed from the start. Google Drive is being built first and is the only one that can be signed in to; uploading is not written yet, so none of them takes a shot.",
  providerNotAvailable: "Not available yet",

  signInHeading: "Google Drive",
  signInNote:
    "Signing in lets Driveshot add files to your Drive, and reach nothing else in it. Uploading is not built yet, so this only proves the connection.",
  signInConnect: "Sign in to Google Drive",
  signInWorking: "Waiting for your browser\u2026",
  signInDisconnect: "Sign out",
  signInNone: "Not signed in.",
  signInHeld: (when: string) => `Signed in. The connection lasts until ${when}.`,
  signInRenews: "It renews itself after that, without asking again.",
  signInExpiresForGood:
    "After that you will be asked to sign in again, because Google issued no renewal for this sign-in.",
  signInExpired: "The connection has run out. Sign in again.",
  signInUnavailable: "Whether Driveshot is signed in could not be read.",
  signInUsingBuiltInClient: "Using the Google client built into Driveshot.",
  signInUsingOwnClient: "Using the Google client you supplied.",
  signInOnlyInMemory:
    "This lasts until Driveshot is closed. Keeping it across restarts is not built yet.",

  clientSummary: "Use a Google client of your own",
  clientNote:
    "Driveshot signs in through its own Google client. Putting your own here makes it depend on your Google Cloud project instead of this one, which is what to do if the built-in one ever stops working. Create it in the Google Cloud console as a Desktop app.",
  clientIdLabel: "Client ID",
  clientSecretLabel: "Client secret",
  clientSecretKept: "Saved. Leave blank to keep it.",
  clientSecretNone: "Optional",
  clientSave: "Save",
  clientForget: "Use the built-in client again",
  clientNoneSaved: "No client of your own is saved, so the built-in one is used.",
  clientSaved: (path: string) => `Saved to ${path}. Sign in again to use it.`,
  clientForgotten: "Removed. The built-in client is used again.",
  clientWhereItLives: (path: string) => `It is kept in ${path}.`,
  clientFailed: "The Google client could not be saved.",

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
  captureWhatItDoes:
    "Pressing it dims every screen. Drag a rectangle on one of them and that part is saved to your pictures folder; nothing is uploaded yet.",

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

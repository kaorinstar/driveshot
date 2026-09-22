# Security policy

[日本語版はこちら / Japanese version](SECURITY.ja.md)

## Reporting a vulnerability

Report it privately, through GitHub's own form:

https://github.com/kaorinstar/driveshot/security/advisories/new

Please do not open an ordinary issue for one. An issue is public from the moment it is written,
and it cannot be taken back.

A report is most useful when it says what the problem is, what someone gains from it, and the
steps that reproduce it. The version of Windows or macOS and the version of Driveshot both help.

One person maintains this project, so a reply is not immediate. Expect one within seven business
days. If none has arrived by then the report has been missed rather than turned down, so comment
on the advisory to raise it again.

## Which versions are supported

The newest release, and no other. A fix reaches a user by replacing the installed application
rather than by patching a release already out. The releases are listed at
https://github.com/kaorinstar/driveshot/releases.

## What the application does today

This is the ground a report is judged against, so it is written out here rather than left to be
read from the source. **Driveshot has not been released yet**, and what it does today is far less
than what it is being built to do. Both are written out: the section below is the application as
it stands, and the one after it is what is coming, so that a design can be questioned before it
ships rather than after.

- It runs as a normal user process. It asks for no elevation, installs no service, and creates no
  scheduled task.
- It captures a region of the screen and writes the image to the user's pictures folder. This
  needs no network and no account, and it keeps working whatever happens to the rest.
- **It signs in to Google Drive, and that is the only network connection it makes.** Two hosts,
  both Google's: the browser is sent to `accounts.google.com`, and the application itself makes
  one HTTPS request to `oauth2.googleapis.com` to exchange the resulting code for tokens. It
  uploads nothing, reads nothing from any Drive, and contacts nothing else at all.
  - The sign-in uses PKCE (RFC 7636) and the loopback redirect of RFC 8252. While it is running,
    and only then, Driveshot listens on a port on `127.0.0.1` that the operating system picks. A
    request arriving there is acted on only if it carries the `state` value that sign-in sent out;
    anything else is answered and ignored, and the port is given up after three minutes.
  - The scope asked for is `drive.file` and nothing else. If less than that is granted, the
    sign-in is refused rather than half-accepted.
- **The tokens are held in memory only.** They go when Driveshot does, so signing in is done once
  per run. Keeping them across restarts means choosing where they live and what protects them,
  and that decision is described here before the change that makes it.
- It reads one file of its own, and only if it exists: `google-client.json` in its configuration
  folder, which holds an OAuth client of the user's own to use in place of the built-in one. It
  writes no file of its own. Settings that persist are
  [#9](https://github.com/kaorinstar/driveshot/issues/9).
- It opens one window, which shows the cloud drives, the retention setting, the capture key, and
  the sign-in. Uploading is not built, so no shot has ever left the machine.

## What it is being built to do

Every item here is a thing to look at in a security review, and none of it is implemented yet.

- **Capture part of the screen.** On macOS this needs the "Screen Recording" permission, which
  the user grants in System Settings; there is no way around it and Driveshot will not try to
  find one. On Windows no permission is needed.
- **Upload to a cloud drive the user owns**, over OAuth. Google Drive comes first and on its own
  ([#2](https://github.com/kaorinstar/driveshot/issues/2)); OneDrive and Dropbox follow as
  [#8](https://github.com/kaorinstar/driveshot/issues/8). The application will hold an access
  token and a refresh token for whichever drive is connected. Where those tokens are stored, and
  what they are protected with, is a decision still to be made
  ([#6](https://github.com/kaorinstar/driveshot/issues/6)) and will be described here before the
  release that makes it.
- **Ask Google for one permission and no more.** The scope is `drive.file`, which reaches the
  files Driveshot itself created and nothing else in the user's Drive. A token taken from a
  machine cannot read the rest of that Drive, because the permission to do so was never asked
  for. Widening this would be a change to report here, not an implementation detail.
- **Publish a share link**, which means setting the uploaded file to "anyone with the link can
  view" at the provider. A link like that is a secret in the weak sense: anyone who has it can see
  the image. That is the point of the feature, and it is also its main risk, which is why the
  retention below is not optional extra.
- **Delete the file when its retention runs out.** Driveshot deletes only files it uploaded and
  has a record of. A file the record does not name is not Driveshot's to delete.
- **Keep a record of every upload**, locally: which drive, which file, when it was uploaded, and
  how long it is kept. That record is what makes deletion possible without asking the provider
  about every file.

## What one Google Cloud project carries, and what it does not

Driveshot ships with an OAuth client identifier. It belongs to a Google Cloud project owned by
one account, and that is a single point of failure worth stating plainly rather than leaving for
somebody to find.

**If that account or project is lost**, nobody can sign in and refresh tokens already issued stop
working. Images already uploaded are not affected - they are in each user's own Drive, and the
share links keep working - but Driveshot can no longer delete them when their retention runs out.
Files meant to stop existing would stay, still readable by whoever holds the link. That is a
failure of the feature this application is for, so it is treated as one.

**What it is not** is a disclosure. There is no server here holding images or tokens: the images
go straight to the user's own Drive, and the tokens stay on the user's own machine. Losing the
project costs automation, not confidentiality, and there is no store for anyone to break into.

**What bounds it.** Settings will carry a client identifier and secret of the user's own; left
empty, the built-in one is used. A user who fills them in depends on no project but their own. On
Google Workspace that client can be "Internal", which needs no verification and no published
pages. On a personal account it can stay in "Testing", which needs none either, at the cost of
signing in again every seven days. `docs/architecture.md` has all three routes written out.

**Capture does not sit behind any of this.** Taking a shot and saving it to disk uses no network
and no account, and a failed upload leaves the image on disk. Whatever happens to a drive, a
sign-in or a client, a screenshot can still be taken and saved.

## What is already known

These are documented. A report of one tells us nothing that is not already written down.

- **Nothing is code-signed.** The Windows installer is not signed, so SmartScreen warns the first
  time it is run. The macOS application is neither signed nor notarized, so macOS refuses to open
  it until the user allows it through System Settings. Certificates are the fix for both and
  neither has been bought
  ([#12](https://github.com/kaorinstar/driveshot/issues/12)).
- **A share link is public to whoever holds it.** "Anyone with the link can view" is what
  publishing one means at every provider. Driveshot's answer to that is the retention: the file
  stops existing, so the link stops working.
- **Retention only runs while Driveshot does.** The application deletes expired files when it is
  running. A machine that stays off past a file's retention leaves that file in place until the
  next time Driveshot starts. Whether that is acceptable, or whether deletion needs to run
  somewhere that is always on, is the open question in
  [#3](https://github.com/kaorinstar/driveshot/issues/3) and
  [docs/architecture.md](docs/architecture.md).

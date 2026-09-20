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
- **It makes no network connection of any kind**, because none of the cloud code exists yet.
- It reads no file of its own and writes none. There are no settings to save yet.
- It opens one window, which shows a list of cloud drives and a retention setting. Neither does
  anything beyond being displayed.

## What it is being built to do

Every item here is a thing to look at in a security review, and none of it is implemented yet.

- **Capture part of the screen.** On macOS this needs the "Screen Recording" permission, which
  the user grants in System Settings; there is no way around it and Driveshot will not try to
  find one. On Windows no permission is needed.
- **Upload to a cloud drive the user owns**, over OAuth, to Google Drive, OneDrive or Dropbox.
  The application will hold an access token and a refresh token for whichever drive is connected.
  Where those tokens are stored, and what they are protected with, is a decision still to be made
  and will be described here before the release that makes it.
- **Publish a share link**, which means setting the uploaded file to "anyone with the link can
  view" at the provider. A link like that is a secret in the weak sense: anyone who has it can see
  the image. That is the point of the feature, and it is also its main risk, which is why the
  retention below is not optional extra.
- **Delete the file when its retention runs out.** Driveshot deletes only files it uploaded and
  has a record of. A file the record does not name is not Driveshot's to delete.
- **Keep a record of every upload**, locally: which drive, which file, when it was uploaded, and
  how long it is kept. That record is what makes deletion possible without asking the provider
  about every file.

## What is already known

These are documented. A report of one tells us nothing that is not already written down.

- **Nothing is code-signed.** The Windows installer is not signed, so SmartScreen warns the first
  time it is run. The macOS application is neither signed nor notarized, so macOS refuses to open
  it until the user allows it through System Settings. Certificates are the fix for both and
  neither has been bought.
- **A share link is public to whoever holds it.** "Anyone with the link can view" is what
  publishing one means at every provider. Driveshot's answer to that is the retention: the file
  stops existing, so the link stops working.
- **Retention only runs while Driveshot does.** The application deletes expired files when it is
  running. A machine that stays off past a file's retention leaves that file in place until the
  next time Driveshot starts. Whether that is acceptable, or whether deletion needs to run
  somewhere that is always on, is the open question in
  [docs/architecture.md](docs/architecture.md).

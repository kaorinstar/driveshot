---
description: Merge the current main into this branch and confirm the result still builds
allowed-tools: Bash(git:*), Bash(cargo:*), Bash(npm:*), Bash(sh:*), Read, Edit, Grep, Glob
---

Bring this branch up to date with `main`. Several sessions write on this repository at once, so
`main` moves while a branch is being written; merging it in now is cheaper than meeting every
change at once at the end.

Work in this order and stop at the first step that fails.

1. Report where you are: `git status --short --branch`. If the working tree has changes that are
   not committed, commit them first. Do not stash them — a stash is easy to leave behind.
2. `git fetch origin main`
3. `git rev-list --count HEAD..FETCH_HEAD`. If it is `0` the branch is already up to date. Say so
   and stop.
4. Say what is coming in before merging it: `git log --oneline HEAD..FETCH_HEAD`, and
   `git diff --name-only $(git merge-base HEAD FETCH_HEAD) FETCH_HEAD`.
5. `git merge --no-edit FETCH_HEAD`

   Merge, never rebase. A rebase rewrites commits that another session or a pull request may
   already have, and this branch is not private.
6. If the merge conflicts, resolve it by hand:
   - Keep both sides' intent. A conflict means two sessions changed the same lines, so deleting
     one side silently undoes work that is already on `main`.
   - `version.md` and `version.ja.md` conflict under `## Unreleased` by addition rather than by
     disagreement: when both sides added an entry, keep both, in both files, and keep the two
     files saying the same thing.
   - `src/strings.ts` conflicts the same way when both sides added a string. Keep both.
   - `Cargo.lock` is generated. Do not edit it by hand: take either side, then run
     `cargo check --workspace` (or `cargo metadata --format-version 1 >/dev/null` where the
     application crate cannot be built) and commit what that writes.
   - `package-lock.json` is the same. Take either side and run `npm install` to rewrite it.
   - Line endings are LF, except `.bat`, `.cmd` and `.ps1`. Do not let a resolution change them.
   - Then `git add <files>` and `git commit --no-edit`.
   - If a conflict cannot be resolved without guessing which behaviour was intended, stop, leave
     the merge in progress, and ask. Do not pick a side to get the build green.
7. Build and test the merged result, because two changes that are each correct can still be
   wrong together:

   ```
   cargo fmt --all -- --check
   cargo clippy -p driveshot-core --all-targets -- -D warnings
   cargo test -p driveshot-core
   npm run build
   ```

   On Windows or macOS, run the whole workspace instead of `driveshot-core` alone:

   ```
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   ```

   On Linux those two need the WebKitGTK development packages
   (`libwebkit2gtk-4.1-dev libsoup-3.0-dev libgtk-3-dev librsvg2-dev patchelf`). Without them the
   application crate does not build, which is expected rather than a failure: say so and leave
   those two commands to CI.
8. Report, in this order: how many commits came in, which files conflicted and how each one was
   resolved, and whether the build and the tests passed. Then say what a human still has to check
   by running the application, if the merged commits changed anything a build cannot verify.

// The settings window.
//
// It holds no logic of its own: which drives exist and when a shot falls due for deletion are
// both answered by the Rust side, which asks driveshot-core. The window draws the answer. Keep
// that split - a calculation written here is a calculation with no test over it.

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { strings } from "./strings";

/** One cloud drive, as `providers` in src-tauri/src/main.rs returns it. */
interface ProviderInfo {
  id: string;
  name: string;
  available: boolean;
}

/** The answer from `expiry_preview` in src-tauri/src/main.rs. */
interface ExpiryPreview {
  expires_at: string | null;
}

/** The answer from `hotkey_status` in src-tauri/src/main.rs. */
interface HotkeyStatus {
  shortcut: string;
  registered: boolean;
  error: string | null;
  last_fired: string | null;
}

/** The answer from `sign_in_status` in src-tauri/src/signin.rs. */
interface SignInStatus {
  signed_in: boolean;
  expires_at: string | null;
  can_refresh: boolean;
  client_source: "built-in" | "user-supplied" | null;
  client_problem: string | null;
  client_file: string | null;
  saved_client_id: string | null;
}

/** The retention periods the window offers. `null` means the shot is kept until deleted by hand. */
const RETENTION_CHOICES: readonly (number | null)[] = [1, 7, 30, 90, null];

function element<T extends HTMLElement>(id: string): T {
  const found = document.getElementById(id);
  if (!found) {
    throw new Error(`index.html has no element with the id '${id}'`);
  }
  return found as T;
}

function writeStaticText(): void {
  element("subtitle").textContent = strings.subtitle;
  element("destination-heading").textContent = strings.destinationHeading;
  element("destination-note").textContent = strings.destinationNote;
  element("sign-in-heading").textContent = strings.signInHeading;
  element("sign-in-note").textContent = strings.signInNote;
  element("sign-in-button").textContent = strings.signInConnect;
  element("sign-out-button").textContent = strings.signInDisconnect;
  element("client-summary").textContent = strings.clientSummary;
  element("client-note").textContent = strings.clientNote;
  element("client-id-label").textContent = strings.clientIdLabel;
  element("client-secret-label").textContent = strings.clientSecretLabel;
  element("client-save").textContent = strings.clientSave;
  element("client-forget").textContent = strings.clientForget;
  element("last-shot-heading").textContent = strings.lastShotHeading;
  element("hotkey-heading").textContent = strings.hotkeyHeading;
  element("hotkey-note").textContent = strings.hotkeyNote;
  element("retention-heading").textContent = strings.retentionHeading;
  element("retention-note").textContent = strings.retentionNote;
  element("retention-label").textContent = strings.retentionLabel;
}

async function drawProviders(): Promise<void> {
  const providers = await invoke<ProviderInfo[]>("providers");
  const list = element<HTMLUListElement>("providers");
  list.replaceChildren();

  for (const provider of providers) {
    const item = document.createElement("li");
    item.className = "providers__item";

    const name = document.createElement("span");
    name.className = "providers__name";
    name.textContent = provider.name;
    item.append(name);

    if (!provider.available) {
      const state = document.createElement("span");
      state.className = "providers__state";
      state.textContent = strings.providerNotAvailable;
      item.append(state);
    }

    list.append(item);
  }
}

/**
 * Puts the saved client back into the fields under the sign-in card.
 *
 * The identifier is filled in; the secret never is, because the Rust side does not return it. An
 * empty secret field means "keep what is stored", which is what its placeholder says.
 */
function drawClient(status: SignInStatus): void {
  const id = element<HTMLInputElement>("client-id");
  const secret = element<HTMLInputElement>("client-secret");
  const forget = element<HTMLButtonElement>("client-forget");
  const own = status.saved_client_id;

  // Only while the user is not part-way through typing one: redrawing on every window focus
  // would otherwise undo what they had half entered.
  if (document.activeElement !== id) {
    id.value = own ?? "";
  }
  secret.placeholder =
    own === null ? strings.clientSecretNone : strings.clientSecretKept;
  forget.hidden = own === null;

  const where = element("client-result");
  if (where.textContent === "" || where.textContent === null) {
    where.textContent =
      own === null
        ? strings.clientNoneSaved
        : status.client_file === null
          ? ""
          : strings.clientWhereItLives(status.client_file);
  }
}

/** Draws what `sign_in_status` answered: the connection, then which client it would use. */
function drawSignIn(status: SignInStatus): void {
  const result = element("sign-in");
  const connect = element<HTMLButtonElement>("sign-in-button");
  const disconnect = element<HTMLButtonElement>("sign-out-button");

  drawClient(status);

  // A client problem is the thing to say first. Without one there is nothing to sign in with, and
  // the Rust side's sentence names the file to write and what to put in it.
  if (status.client_problem !== null) {
    result.textContent = status.client_problem;
    connect.disabled = true;
    disconnect.hidden = true;
    return;
  }

  connect.disabled = false;
  const client =
    status.client_source === "user-supplied"
      ? strings.signInUsingOwnClient
      : strings.signInUsingBuiltInClient;

  if (!status.signed_in) {
    // An expiry that has passed is not the same as never having signed in, and saying which is
    // the difference between "press the button" and "something went wrong".
    result.textContent = `${
      status.expires_at === null ? strings.signInNone : strings.signInExpired
    } ${client}`;
    connect.textContent = strings.signInConnect;
    disconnect.hidden = true;
    return;
  }

  const until = new Date(status.expires_at ?? "").toLocaleString();
  const after = status.can_refresh
    ? strings.signInRenews
    : strings.signInExpiresForGood;
  result.textContent = `${strings.signInHeld(until)} ${after} ${strings.signInOnlyInMemory} ${client}`;
  connect.textContent = strings.signInConnect;
  disconnect.hidden = false;
}

async function showSignIn(): Promise<void> {
  try {
    drawSignIn(await invoke<SignInStatus>("sign_in_status"));
  } catch {
    element("sign-in").textContent = strings.signInUnavailable;
  }
}

/** Runs the sign-in, which opens a browser and waits there until the user comes back. */
async function signIn(): Promise<void> {
  const connect = element<HTMLButtonElement>("sign-in-button");
  connect.disabled = true;
  connect.textContent = strings.signInWorking;

  try {
    drawSignIn(await invoke<SignInStatus>("sign_in"));
  } catch (problem) {
    // Every failure the Rust side returns here is one the user can act on, and it names what: a
    // consent screen they cancelled, a client that no longer exists, a network that was not there.
    element("sign-in").textContent = String(problem);
    connect.textContent = strings.signInConnect;
  } finally {
    connect.disabled = false;
  }
}

async function showLastShot(): Promise<void> {
  const result = element("last-shot");

  try {
    // The Rust side puts either a path or a reason in here: a capture that failed is a thing the
    // user pressed a key for and is entitled to an answer about, and this window is where it
    // appears, because Driveshot has no window on screen the rest of the time.
    const last = await invoke<string | null>("last_shot");
    result.textContent =
      last === null ? strings.lastShotNone : strings.lastShotSaved(last);
  } catch {
    result.textContent = strings.lastShotUnavailable;
  }
}

async function showHotkey(): Promise<void> {
  const result = element("hotkey");

  try {
    const status = await invoke<HotkeyStatus>("hotkey_status");

    // A key another application already holds leaves Driveshot with one that does nothing, which
    // is why the Rust side reports the reason rather than assuming it worked. Its sentence is
    // shown as it is: it names the combination and what refused it.
    const held = status.registered
      ? strings.hotkeyHeld(status.shortcut)
      : (status.error ?? strings.hotkeyUnavailable);

    const pressed =
      status.last_fired === null
        ? strings.hotkeyNeverPressed
        : strings.hotkeyLastPressed(new Date(status.last_fired).toLocaleString());

    result.textContent = `${held} ${pressed} ${strings.captureWhatItDoes}`;
  } catch {
    result.textContent = strings.hotkeyUnavailable;
  }
}

function fillRetentionChoices(): void {
  const select = element<HTMLSelectElement>("retention");
  select.replaceChildren();

  for (const choice of RETENTION_CHOICES) {
    const option = document.createElement("option");
    // The value carries the choice back: "forever", or the number of days as text.
    option.value = choice === null ? "forever" : String(choice);
    option.textContent =
      choice === null ? strings.retentionForever : strings.retentionDays(choice);
    select.append(option);
  }

  // Seven days is the default: long enough for a link shared in a chat to still work the next
  // working day, short enough that a drive does not fill with shots nobody looks at again.
  select.value = "7";
}

async function showExpiry(): Promise<void> {
  const select = element<HTMLSelectElement>("retention");
  const result = element("expiry");
  const days = select.value === "forever" ? null : Number(select.value);

  try {
    const preview = await invoke<ExpiryPreview>("expiry_preview", { days });
    result.textContent =
      preview.expires_at === null
        ? strings.expiryForever
        : strings.expiryAt(new Date(preview.expires_at).toLocaleString());
  } catch {
    // The Rust side refuses a retention it cannot honour - zero days, for one - and says so.
    // Showing the window's own sentence keeps that message in the strings table with the rest.
    result.textContent = strings.expiryFailed;
  }
}

async function start(): Promise<void> {
  writeStaticText();
  fillRetentionChoices();
  element<HTMLSelectElement>("retention").addEventListener("change", () => {
    void showExpiry();
  });
  element("sign-in-button").addEventListener("click", () => {
    void signIn();
  });
  element("client-save").addEventListener("click", () => {
    void (async () => {
      const id = element<HTMLInputElement>("client-id");
      const secret = element<HTMLInputElement>("client-secret");
      try {
        const status = await invoke<SignInStatus>("save_google_client", {
          clientId: id.value,
          clientSecret: secret.value,
        });
        // The secret is never read back, so the field is emptied rather than left holding a value
        // that a later save would send again.
        secret.value = "";
        element("client-result").textContent =
          status.client_file === null
            ? strings.clientFailed
            : strings.clientSaved(status.client_file);
        drawSignIn(status);
      } catch (problem) {
        element("client-result").textContent = String(problem);
      }
    })();
  });
  element("client-forget").addEventListener("click", () => {
    void (async () => {
      try {
        const status = await invoke<SignInStatus>("forget_google_client");
        element<HTMLInputElement>("client-secret").value = "";
        element("client-result").textContent = strings.clientForgotten;
        drawSignIn(status);
      } catch (problem) {
        element("client-result").textContent = String(problem);
      }
    })();
  });
  element("sign-out-button").addEventListener("click", () => {
    void (async () => {
      try {
        drawSignIn(await invoke<SignInStatus>("sign_out"));
      } catch {
        element("sign-in").textContent = strings.signInUnavailable;
      }
    })();
  });

  // The Rust side brings this window up when the key is pressed. If it was already open, nothing
  // reloads it, so the time it shows would be the one it read when it opened.
  await listen("capture-requested", () => {
    void showHotkey();
  });

  // The window is hidden rather than closed, so it is the same page every time it comes back.
  // A shot taken while it was away, or a failure that brought it back, has to be read again.
  window.addEventListener("focus", () => {
    void showLastShot();
    void showHotkey();
    void showSignIn();
  });

  await Promise.all([
    drawProviders(),
    showExpiry(),
    showHotkey(),
    showLastShot(),
    showSignIn(),
  ]);
}

void start();

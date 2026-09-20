// The settings window.
//
// It holds no logic of its own: which drives exist and when a shot falls due for deletion are
// both answered by the Rust side, which asks driveshot-core. The window draws the answer. Keep
// that split - a calculation written here is a calculation with no test over it.

import { invoke } from "@tauri-apps/api/core";
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
  await Promise.all([drawProviders(), showExpiry()]);
}

void start();

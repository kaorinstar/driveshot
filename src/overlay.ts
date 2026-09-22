// The selection overlay: one of these runs on each monitor while a shot is being taken.
//
// It draws a rectangle and hands the result to the Rust side. Nothing here decides which pixels
// that rectangle covers - that is driveshot-core's `pixels_for`, working from the size of this
// page and the size of the captured image, because the two platforms disagree about what a
// monitor's own width means.
//
// This page is not loaded afresh for each shot. Its window is built once, at startup, and kept,
// because building a web view per monitor took about a second and that second was the whole delay
// between pressing the key and the screen dimming (#23). So the page still holds the last
// capture's state when the next one starts, and putting that back is what `restart` below is for.

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { strings } from "./strings";

/**
 * How far the pointer has to move before a drag counts as one, in points.
 *
 * Below this it is a click: somebody dismissing the overlay, or a hand that moved while pressing.
 * A one-pixel screenshot is never what was wanted, and the geometry in the core crate deliberately
 * leaves this decision here, where the pointer is.
 */
const MINIMUM_DRAG = 4;

/** Where the drag started, in this page's coordinates. */
interface Point {
  x: number;
  y: number;
}

function element<T extends HTMLElement>(id: string): T {
  const found = document.getElementById(id);
  if (!found) {
    throw new Error(`overlay.html has no element with the id '${id}'`);
  }
  return found as T;
}

/** Which monitor this overlay covers, from the query string its window was opened with. */
function monitorIndex(): number {
  const raw = new URLSearchParams(window.location.search).get("monitor");
  const index = Number(raw);
  return Number.isInteger(index) && index >= 0 ? index : 0;
}

function start(): void {
  const hint = element("hint");
  const box = element("selection");
  const monitor = monitorIndex();

  hint.textContent = strings.overlayHint;

  let origin: Point | null = null;
  // Once a capture has been asked for, this page is on its way out. A second mouseup, or an
  // Escape arriving after it, must not cancel the capture that is already happening.
  let finished = false;

  const cancel = (): void => {
    if (finished) {
      return;
    }
    finished = true;
    void invoke("cancel_capture");
  };

  const draw = (from: Point, to: Point): void => {
    const left = Math.min(from.x, to.x);
    const top = Math.min(from.y, to.y);
    box.style.left = `${left}px`;
    box.style.top = `${top}px`;
    box.style.width = `${Math.abs(to.x - from.x)}px`;
    box.style.height = `${Math.abs(to.y - from.y)}px`;
    box.hidden = false;
  };

  window.addEventListener("mousedown", (event: MouseEvent) => {
    if (finished || event.button !== 0) {
      return;
    }
    origin = { x: event.clientX, y: event.clientY };
    // The hint would otherwise sit inside a selection drawn near the top of the screen, and the
    // shot would have it in the corner.
    hint.hidden = true;
    draw(origin, origin);
  });

  window.addEventListener("mousemove", (event: MouseEvent) => {
    if (origin === null || finished) {
      return;
    }
    draw(origin, { x: event.clientX, y: event.clientY });
  });

  window.addEventListener("mouseup", (event: MouseEvent) => {
    if (origin === null || finished || event.button !== 0) {
      return;
    }

    const from = origin;
    origin = null;
    box.hidden = true;

    const width = event.clientX - from.x;
    const height = event.clientY - from.y;

    if (Math.abs(width) < MINIMUM_DRAG || Math.abs(height) < MINIMUM_DRAG) {
      cancel();
      return;
    }

    finished = true;
    // The Rust side hides every overlay, takes the shot and saves it. A failure there opens the
    // settings window and says why, so nothing is reported from this page.
    void invoke("finish_capture", {
      monitor,
      selection: { x: from.x, y: from.y, width, height },
    });
  });

  window.addEventListener("keydown", (event: KeyboardEvent) => {
    if (event.key === "Escape") {
      cancel();
    }
  });

  // A right click is not a selection, and on some platforms it would open a context menu over the
  // overlay instead.
  window.addEventListener("contextmenu", (event: MouseEvent) => {
    event.preventDefault();
    cancel();
  });

  // Every capture starts here. The window is invisible until `overlay_ready` answers, so what
  // reaches the screen is this page as it began rather than as the last shot left it.
  //
  // Nothing waits for an animation frame before answering, and this is the one place where that
  // is safe: the page has been drawn since it loaded, and what changes below is two hidden
  // attributes on elements that are already laid out. Showing a window whose page has drawn
  // nothing at all is the white flash of #20, and that moment was over long before a key was
  // pressed - this window was built at startup.
  const restart = (): void => {
    origin = null;
    finished = false;
    box.hidden = true;
    hint.hidden = false;
    void invoke("overlay_ready");
  };

  void listen("capture-begin", restart);

  // An overlay built while a capture is already running - a monitor plugged in since the last one
  // - was not listening when that event went out, and does not need to be: it has never held a
  // selection to clear. It answers once it has drawn itself instead. Two nested animation frames
  // is the usual way to wait for "has actually painted": the first runs before the coming frame is
  // rendered, the second after it has been. At startup, where there is no capture running, the
  // Rust side ignores this.
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      void invoke("overlay_ready");
    });
  });
}

start();

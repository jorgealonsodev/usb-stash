/**
 * AutoLockTimer — tracks user inactivity and calls `onLock` when the configured
 * timeout elapses.
 *
 * Listens for `mousemove`, `keydown`, `mousedown`, `touchstart`, and `scroll`
 * events on `window`. Each event resets the internal `setTimeout`.
 *
 * Usage:
 * ```ts
 * const timer = createAutoLockTimer(300, () => lockStash());
 * timer.start();
 * // ... later
 * timer.stop();
 * ```
 */

export interface AutoLockTimer {
  start: () => void;
  stop: () => void;
  reset: () => void;
}

const EVENTS = ["mousemove", "keydown", "mousedown", "touchstart", "scroll"];

export function createAutoLockTimer(
  timeoutSeconds: number,
  onLock: () => void,
): AutoLockTimer {
  let timerId: ReturnType<typeof setTimeout> | null = null;
  let active = false;
  let handler: (() => void) | null = null;

  function schedule() {
    if (timerId !== null) {
      clearTimeout(timerId);
    }
    if (timeoutSeconds <= 0) return; // off
    timerId = setTimeout(() => {
      timerId = null;
      onLock();
    }, timeoutSeconds * 1000);
  }

  function onActivity() {
    schedule();
  }

  function start() {
    if (active) return;
    active = true;
    handler = onActivity;
    for (const event of EVENTS) {
      window.addEventListener(event, handler, { passive: true });
    }
    schedule();
  }

  function stop() {
    if (!active) return;
    active = false;
    if (handler) {
      for (const event of EVENTS) {
        window.removeEventListener(event, handler);
      }
    }
    if (timerId !== null) {
      clearTimeout(timerId);
      timerId = null;
    }
  }

  function reset() {
    if (active) {
      schedule();
    }
  }

  return { start, stop, reset };
}

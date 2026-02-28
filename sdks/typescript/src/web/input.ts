/**
 * Browser input handler for the GoudEngine web backend.
 *
 * Listens to keyboard, mouse, and wheel events on a target element
 * (typically the `<canvas>`) and forwards them to the Wasm game
 * instance via its setter methods.
 */

/** Wasm game interface — only the input-related subset. */
interface WasmInputSink {
  press_key(keyCode: number): void;
  release_key(keyCode: number): void;
  press_mouse_button(button: number): void;
  release_mouse_button(button: number): void;
  set_mouse_position(x: number, y: number): void;
  add_scroll_delta(dx: number, dy: number): void;
}

/**
 * Maps `KeyboardEvent.code` strings to integer key codes matching the
 * values used on the Rust side. The mapping intentionally mirrors the
 * legacy `KeyboardEvent.keyCode` constants for wide compatibility.
 */
const CODE_MAP: Record<string, number> = {
  Space: 32,
  Quote: 222,
  Comma: 188,
  Minus: 189,
  Period: 190,
  Slash: 191,
  Digit0: 48, Digit1: 49, Digit2: 50, Digit3: 51, Digit4: 52,
  Digit5: 53, Digit6: 54, Digit7: 55, Digit8: 56, Digit9: 57,
  Semicolon: 186,
  Equal: 187,
  KeyA: 65, KeyB: 66, KeyC: 67, KeyD: 68, KeyE: 69,
  KeyF: 70, KeyG: 71, KeyH: 72, KeyI: 73, KeyJ: 74,
  KeyK: 75, KeyL: 76, KeyM: 77, KeyN: 78, KeyO: 79,
  KeyP: 80, KeyQ: 81, KeyR: 82, KeyS: 83, KeyT: 84,
  KeyU: 85, KeyV: 86, KeyW: 87, KeyX: 88, KeyY: 89,
  KeyZ: 90,
  BracketLeft: 219,
  Backslash: 220,
  BracketRight: 221,
  Backquote: 192,
  Escape: 27,
  Enter: 13,
  Tab: 9,
  Backspace: 8,
  Insert: 45,
  Delete: 46,
  ArrowRight: 39,
  ArrowLeft: 37,
  ArrowDown: 40,
  ArrowUp: 38,
  PageUp: 33,
  PageDown: 34,
  Home: 36,
  End: 35,
  CapsLock: 20,
  ScrollLock: 145,
  NumLock: 144,
  PrintScreen: 44,
  Pause: 19,
  F1: 112, F2: 113, F3: 114, F4: 115, F5: 116, F6: 117,
  F7: 118, F8: 119, F9: 120, F10: 121, F11: 122, F12: 123,
  Numpad0: 96, Numpad1: 97, Numpad2: 98, Numpad3: 99, Numpad4: 100,
  Numpad5: 101, Numpad6: 102, Numpad7: 103, Numpad8: 104, Numpad9: 105,
  NumpadDecimal: 110,
  NumpadDivide: 111,
  NumpadMultiply: 106,
  NumpadSubtract: 109,
  NumpadAdd: 107,
  NumpadEnter: 13,
  ShiftLeft: 16, ShiftRight: 16,
  ControlLeft: 17, ControlRight: 17,
  AltLeft: 18, AltRight: 18,
  MetaLeft: 91, MetaRight: 93,
};

export function codeToKeyCode(code: string): number | undefined {
  return CODE_MAP[code];
}

/**
 * Attaches browser event listeners and pipes them into the wasm game.
 *
 * Returns a teardown function that removes all listeners.
 */
export function attachInputHandlers(
  target: HTMLElement,
  sink: WasmInputSink,
): () => void {
  const onKeyDown = (e: KeyboardEvent) => {
    const kc = codeToKeyCode(e.code);
    if (kc !== undefined) {
      e.preventDefault();
      sink.press_key(kc);
    }
  };

  const onKeyUp = (e: KeyboardEvent) => {
    const kc = codeToKeyCode(e.code);
    if (kc !== undefined) {
      e.preventDefault();
      sink.release_key(kc);
    }
  };

  const onMouseDown = (e: MouseEvent) => {
    sink.press_mouse_button(e.button);
  };

  const onMouseUp = (e: MouseEvent) => {
    sink.release_mouse_button(e.button);
  };

  const onMouseMove = (e: MouseEvent) => {
    const rect = target.getBoundingClientRect();
    sink.set_mouse_position(e.clientX - rect.left, e.clientY - rect.top);
  };

  const onWheel = (e: WheelEvent) => {
    e.preventDefault();
    sink.add_scroll_delta(e.deltaX, e.deltaY);
  };

  const onContextMenu = (e: Event) => e.preventDefault();

  window.addEventListener('keydown', onKeyDown);
  window.addEventListener('keyup', onKeyUp);
  target.addEventListener('mousedown', onMouseDown);
  target.addEventListener('mouseup', onMouseUp);
  target.addEventListener('mousemove', onMouseMove);
  target.addEventListener('wheel', onWheel, { passive: false });
  target.addEventListener('contextmenu', onContextMenu);

  return () => {
    window.removeEventListener('keydown', onKeyDown);
    window.removeEventListener('keyup', onKeyUp);
    target.removeEventListener('mousedown', onMouseDown);
    target.removeEventListener('mouseup', onMouseUp);
    target.removeEventListener('mousemove', onMouseMove);
    target.removeEventListener('wheel', onWheel);
    target.removeEventListener('contextmenu', onContextMenu);
  };
}

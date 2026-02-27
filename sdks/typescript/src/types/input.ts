/**
 * Input type interfaces for the GoudEngine SDK.
 *
 * These will be expanded when window/input integration is added.
 * For now, they define the planned API surface.
 */

export enum Key {
  Space = 32,
  Escape = 256,
  Enter = 257,
  Tab = 258,
  Backspace = 259,
  Right = 262,
  Left = 263,
  Down = 264,
  Up = 265,
  A = 65,
  D = 68,
  S = 83,
  W = 87,
}

export enum MouseButton {
  Left = 0,
  Right = 1,
  Middle = 2,
}

export interface IInputManager {
  isKeyPressed(key: Key): boolean;
  isKeyJustPressed(key: Key): boolean;
  isKeyJustReleased(key: Key): boolean;
  isMouseButtonPressed(button: MouseButton): boolean;
  getMousePosition(): { x: number; y: number };
  getMouseDelta(): { x: number; y: number };
  getScrollDelta(): { x: number; y: number };
  mapAction(name: string, key: Key): void;
  isActionPressed(name: string): boolean;
  isActionJustPressed(name: string): boolean;
  isActionJustReleased(name: string): boolean;
}

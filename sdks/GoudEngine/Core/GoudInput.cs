using System;
using CsBindgen;

namespace GoudEngine.Core
{
    /// <summary>
    /// Key codes for keyboard input. Values match GLFW key codes.
    /// </summary>
    public static class KeyCode
    {
        public const int Unknown = -1;
        public const int Space = 32;
        public const int Apostrophe = 39;
        public const int Comma = 44;
        public const int Minus = 45;
        public const int Period = 46;
        public const int Slash = 47;
        
        // Numbers
        public const int D0 = 48, D1 = 49, D2 = 50, D3 = 51, D4 = 52;
        public const int D5 = 53, D6 = 54, D7 = 55, D8 = 56, D9 = 57;
        
        // Letters
        public const int A = 65, B = 66, C = 67, D = 68, E = 69, F = 70;
        public const int G = 71, H = 72, I = 73, J = 74, K = 75, L = 76;
        public const int M = 77, N = 78, O = 79, P = 80, Q = 81, R = 82;
        public const int S = 83, T = 84, U = 85, V = 86, W = 87, X = 88;
        public const int Y = 89, Z = 90;
        
        // Function keys
        public const int Escape = 256;
        public const int Enter = 257;
        public const int Tab = 258;
        public const int Backspace = 259;
        public const int Insert = 260;
        public const int Delete = 261;
        public const int Right = 262;
        public const int Left = 263;
        public const int Down = 264;
        public const int Up = 265;
        public const int PageUp = 266;
        public const int PageDown = 267;
        public const int Home = 268;
        public const int End = 269;
        
        public const int F1 = 290, F2 = 291, F3 = 292, F4 = 293;
        public const int F5 = 294, F6 = 295, F7 = 296, F8 = 297;
        public const int F9 = 298, F10 = 299, F11 = 300, F12 = 301;
        
        // Modifiers
        public const int LeftShift = 340;
        public const int LeftControl = 341;
        public const int LeftAlt = 342;
        public const int LeftSuper = 343;
        public const int RightShift = 344;
        public const int RightControl = 345;
        public const int RightAlt = 346;
        public const int RightSuper = 347;
    }

    /// <summary>
    /// Mouse button codes.
    /// </summary>
    public static class MouseButton
    {
        public const int Left = 0;
        public const int Right = 1;
        public const int Middle = 2;
        public const int Button4 = 3;
        public const int Button5 = 4;
    }

    /// <summary>
    /// Provides input query functions for a GoudWindow.
    /// Input state is updated automatically when calling GoudWindow.PollEvents().
    /// </summary>
    public sealed class GoudInput
    {
        private readonly CsBindgen.GoudContextId _contextId;

        /// <summary>
        /// Creates an input handler for the specified window.
        /// </summary>
        /// <param name="window">The window to query input from.</param>
        public GoudInput(GoudWindow window)
        {
            if (window == null) throw new ArgumentNullException(nameof(window));
            _contextId = window.ContextId;
        }

        /// <summary>
        /// Creates an input handler for the specified context ID.
        /// </summary>
        internal GoudInput(CsBindgen.GoudContextId contextId)
        {
            _contextId = contextId;
        }

        // ====================================================================
        // Keyboard
        // ====================================================================

        /// <summary>
        /// Returns true if the specified key is currently pressed.
        /// </summary>
        public bool IsKeyPressed(int key)
        {
            return NativeMethods.goud_input_key_pressed(_contextId, key);
        }

        /// <summary>
        /// Returns true if the key was just pressed this frame.
        /// </summary>
        public bool IsKeyJustPressed(int key)
        {
            return NativeMethods.goud_input_key_just_pressed(_contextId, key);
        }

        /// <summary>
        /// Returns true if the key was just released this frame.
        /// </summary>
        public bool IsKeyJustReleased(int key)
        {
            return NativeMethods.goud_input_key_just_released(_contextId, key);
        }

        // ====================================================================
        // Mouse Buttons
        // ====================================================================

        /// <summary>
        /// Returns true if the specified mouse button is currently pressed.
        /// </summary>
        public bool IsMouseButtonPressed(int button)
        {
            return NativeMethods.goud_input_mouse_button_pressed(_contextId, button);
        }

        /// <summary>
        /// Returns true if the mouse button was just pressed this frame.
        /// </summary>
        public bool IsMouseButtonJustPressed(int button)
        {
            return NativeMethods.goud_input_mouse_button_just_pressed(_contextId, button);
        }

        /// <summary>
        /// Returns true if the mouse button was just released this frame.
        /// </summary>
        public bool IsMouseButtonJustReleased(int button)
        {
            return NativeMethods.goud_input_mouse_button_just_released(_contextId, button);
        }

        // ====================================================================
        // Mouse Position
        // ====================================================================

        /// <summary>
        /// Gets the current mouse position.
        /// </summary>
        public unsafe (float X, float Y) GetMousePosition()
        {
            float x = 0, y = 0;
            NativeMethods.goud_input_get_mouse_position(_contextId, &x, &y);
            return (x, y);
        }

        /// <summary>
        /// Gets the mouse movement delta since the last frame.
        /// </summary>
        public unsafe (float DX, float DY) GetMouseDelta()
        {
            float dx = 0, dy = 0;
            NativeMethods.goud_input_get_mouse_delta(_contextId, &dx, &dy);
            return (dx, dy);
        }

        /// <summary>
        /// Gets the scroll wheel delta since the last frame.
        /// </summary>
        public unsafe (float DX, float DY) GetScrollDelta()
        {
            float dx = 0, dy = 0;
            NativeMethods.goud_input_get_scroll_delta(_contextId, &dx, &dy);
            return (dx, dy);
        }

        // ====================================================================
        // Actions
        // ====================================================================

        /// <summary>
        /// Maps an action name to a keyboard key.
        /// </summary>
        public unsafe bool MapAction(string actionName, int key)
        {
            if (string.IsNullOrEmpty(actionName)) return false;
            
            var bytes = System.Text.Encoding.UTF8.GetBytes(actionName + "\0");
            fixed (byte* namePtr = bytes)
            {
                return NativeMethods.goud_input_map_action_key(_contextId, namePtr, key);
            }
        }

        /// <summary>
        /// Returns true if the specified action is currently pressed.
        /// </summary>
        public unsafe bool IsActionPressed(string actionName)
        {
            if (string.IsNullOrEmpty(actionName)) return false;
            
            var bytes = System.Text.Encoding.UTF8.GetBytes(actionName + "\0");
            fixed (byte* namePtr = bytes)
            {
                return NativeMethods.goud_input_action_pressed(_contextId, namePtr);
            }
        }

        /// <summary>
        /// Returns true if the action was just pressed this frame.
        /// </summary>
        public unsafe bool IsActionJustPressed(string actionName)
        {
            if (string.IsNullOrEmpty(actionName)) return false;
            
            var bytes = System.Text.Encoding.UTF8.GetBytes(actionName + "\0");
            fixed (byte* namePtr = bytes)
            {
                return NativeMethods.goud_input_action_just_pressed(_contextId, namePtr);
            }
        }

        /// <summary>
        /// Returns true if the action was just released this frame.
        /// </summary>
        public unsafe bool IsActionJustReleased(string actionName)
        {
            if (string.IsNullOrEmpty(actionName)) return false;
            
            var bytes = System.Text.Encoding.UTF8.GetBytes(actionName + "\0");
            fixed (byte* namePtr = bytes)
            {
                return NativeMethods.goud_input_action_just_released(_contextId, namePtr);
            }
        }
    }
}

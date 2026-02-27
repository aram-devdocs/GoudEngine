using System;
using System.Runtime.InteropServices;
using CsBindgen;

namespace GoudEngine.Core
{
    /// <summary>
    /// Represents a GoudEngine window for rendering and input handling.
    /// This wraps a windowed context that combines a GoudContext with GLFW window management.
    /// </summary>
    public sealed class GoudWindow : IDisposable
    {
        private CsBindgen.GoudContextId _contextId;
        private bool _disposed;
        private readonly string _title;
        private uint _width;
        private uint _height;

        private const ulong INVALID_CONTEXT = ulong.MaxValue;

        /// <summary>
        /// Gets the native context ID for this window.
        /// </summary>
        public CsBindgen.GoudContextId ContextId => _contextId;

        /// <summary>
        /// Returns true if this window has been disposed.
        /// </summary>
        public bool IsDisposed => _disposed;

        /// <summary>
        /// Gets the window title.
        /// </summary>
        public string Title => _title;

        /// <summary>
        /// Gets the current window width.
        /// </summary>
        public uint Width => _width;

        /// <summary>
        /// Gets the current window height.
        /// </summary>
        public uint Height => _height;

        /// <summary>
        /// Returns true if the window should close (user clicked X or pressed Escape, etc).
        /// </summary>
        public bool ShouldClose
        {
            get
            {
                if (NativeMethods.goud_context_is_valid(_contextId))
                    return NativeMethods.goud_window_should_close(_contextId);
                return true;
            }
        }

        /// <summary>
        /// Creates a new GoudEngine window with the specified dimensions and title.
        /// </summary>
        /// <param name="width">Window width in pixels.</param>
        /// <param name="height">Window height in pixels.</param>
        /// <param name="title">Window title.</param>
        /// <exception cref="Exception">Thrown if window creation fails.</exception>
        public unsafe GoudWindow(uint width, uint height, string title)
        {
            _width = width;
            _height = height;
            _title = title ?? "GoudEngine Window";

            var bytes = System.Text.Encoding.UTF8.GetBytes(_title + "\0");
            fixed (byte* titlePtr = bytes)
            {
                _contextId = NativeMethods.goud_window_create(width, height, titlePtr);
            }

            if (!NativeMethods.goud_context_is_valid(_contextId))
            {
                throw new Exception("Failed to create GoudEngine window");
            }
        }

        /// <summary>
        /// Polls window events and updates input state.
        /// Call this at the beginning of each frame.
        /// </summary>
        /// <returns>The time elapsed since the last frame (delta time) in seconds.</returns>
        public float PollEvents()
        {
            ThrowIfDisposed();
            return NativeMethods.goud_window_poll_events(_contextId);
        }

        /// <summary>
        /// Swaps the front and back buffers, presenting the rendered frame.
        /// Call this at the end of each frame.
        /// </summary>
        public void SwapBuffers()
        {
            ThrowIfDisposed();
            NativeMethods.goud_window_swap_buffers(_contextId);
        }

        /// <summary>
        /// Clears the window with the specified color.
        /// </summary>
        /// <param name="r">Red component (0.0 to 1.0).</param>
        /// <param name="g">Green component (0.0 to 1.0).</param>
        /// <param name="b">Blue component (0.0 to 1.0).</param>
        /// <param name="a">Alpha component (0.0 to 1.0).</param>
        public void Clear(float r = 0.1f, float g = 0.1f, float b = 0.1f, float a = 1.0f)
        {
            ThrowIfDisposed();
            NativeMethods.goud_window_clear(_contextId, r, g, b, a);
        }

        /// <summary>
        /// Gets the current window size.
        /// </summary>
        /// <param name="width">Output: current width in pixels.</param>
        /// <param name="height">Output: current height in pixels.</param>
        /// <returns>True if successful.</returns>
        public unsafe bool GetSize(out uint width, out uint height)
        {
            ThrowIfDisposed();
            
            uint w = 0, h = 0;
            bool result = NativeMethods.goud_window_get_size(_contextId, &w, &h);
            width = w;
            height = h;
            
            if (result)
            {
                _width = w;
                _height = h;
            }
            
            return result;
        }

        /// <summary>
        /// Gets the delta time for the current frame.
        /// </summary>
        /// <returns>Time since last frame in seconds.</returns>
        public float GetDeltaTime()
        {
            ThrowIfDisposed();
            return NativeMethods.goud_window_get_delta_time(_contextId);
        }

        /// <summary>
        /// Sets whether the window should close.
        /// </summary>
        /// <param name="shouldClose">True to mark window for closing.</param>
        public void SetShouldClose(bool shouldClose)
        {
            ThrowIfDisposed();
            NativeMethods.goud_window_set_should_close(_contextId, shouldClose);
        }

        /// <summary>
        /// Validates that this window is not disposed.
        /// </summary>
        private void ThrowIfDisposed()
        {
            if (_disposed)
            {
                throw new ObjectDisposedException(nameof(GoudWindow));
            }
        }

        // ====================================================================
        // IDisposable Implementation
        // ====================================================================

        ~GoudWindow()
        {
            Dispose(false);
        }

        public void Dispose()
        {
            Dispose(true);
            GC.SuppressFinalize(this);
        }

        private void Dispose(bool disposing)
        {
            if (_disposed) return;

            if (NativeMethods.goud_context_is_valid(_contextId))
            {
                NativeMethods.goud_window_destroy(_contextId);
            }

            _disposed = true;
        }

        public override string ToString()
        {
            return _disposed ? "GoudWindow(Disposed)" : $"GoudWindow({_width}x{_height}, \"{_title}\")";
        }
    }
}

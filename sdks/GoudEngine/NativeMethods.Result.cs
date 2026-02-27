// NativeMethods.Result.cs - Manual definition for missing csbindgen struct
// This file contains the Result struct that csbindgen references but doesn't generate
#pragma warning disable CS8500
#pragma warning disable CS8981
using System.Runtime.InteropServices;

namespace CsBindgen
{
    /// <summary>
    /// FFI-safe result type for operations that can fail.
    /// Maps to GoudResult in Rust.
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public unsafe partial struct Result
    {
        /// <summary>
        /// Error code (0 = success, non-zero = error).
        /// </summary>
        public int code;
        
        /// <summary>
        /// True if operation succeeded (code == 0).
        /// </summary>
        [MarshalAs(UnmanagedType.U1)]
        public bool Success;
        
        /// <summary>
        /// Returns true if this result represents success.
        /// </summary>
        public readonly bool IsSuccess => Success && code == 0;
        
        /// <summary>
        /// Returns true if this result represents an error.
        /// </summary>
        public readonly bool IsError => !Success || code != 0;
    }
}

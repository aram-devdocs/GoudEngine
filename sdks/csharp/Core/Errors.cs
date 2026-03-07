using System;
using System.Runtime.InteropServices;
using System.Text;

namespace GoudEngine
{
    /// <summary>
    /// Recovery classification for engine errors.
    /// Matches the Rust RecoveryClass enum values returned by
    /// goud_error_recovery_class.
    /// </summary>
    public enum RecoveryClass
    {
        Recoverable = 0,
        Fatal = 1,
        Degraded = 2,
    }

    /// <summary>
    /// Base exception for all GoudEngine errors. Carries the numeric error
    /// code, human-readable category, subsystem/operation context, and
    /// recovery information retrieved from the Rust FFI layer.
    /// </summary>
    public class GoudException : Exception
    {
        public int ErrorCode { get; }
        public string Category { get; }
        public string Subsystem { get; }
        public string Operation { get; }
        public RecoveryClass Recovery { get; }
        public string RecoveryHint { get; }

        public GoudException(
            int errorCode,
            string message,
            string category,
            string subsystem,
            string operation,
            RecoveryClass recovery,
            string recoveryHint
        ) : base(message)
        {
            ErrorCode = errorCode;
            Category = category;
            Subsystem = subsystem;
            Operation = operation;
            Recovery = recovery;
            RecoveryHint = recoveryHint;
        }

        /// <summary>
        /// Queries all FFI error functions and constructs the appropriate
        /// typed exception subclass based on the error code range.
        /// Returns null if no error is set (code == 0).
        /// </summary>
        public static GoudException? FromLastError()
        {
            int code = NativeMethods.goud_last_error_code();
            if (code == 0)
                return null;

            string message = ReadStringFromFfi(NativeMethods.goud_last_error_message);
            string subsystem = ReadStringFromFfi(NativeMethods.goud_last_error_subsystem);
            string operation = ReadStringFromFfi(NativeMethods.goud_last_error_operation);

            int recoveryRaw = NativeMethods.goud_error_recovery_class(code);
            var recovery = recoveryRaw switch
            {
                1 => RecoveryClass.Fatal,
                2 => RecoveryClass.Degraded,
                _ => RecoveryClass.Recoverable,
            };

            string hint = ReadHintFromFfi(code);
            string category = CategoryFromCode(code);

            return CreateTyped(
                code, message, category, subsystem, operation, recovery, hint
            );
        }

        private static string CategoryFromCode(int code)
        {
            return code switch
            {
                >= 900 => "Internal",
                >= 600 => "Provider",
                >= 500 => "System",
                >= 400 => "Input",
                >= 300 => "Entity",
                >= 200 => "Graphics",
                >= 100 => "Resource",
                >= 1 => "Context",
                _ => "Unknown",
            };
        }

        private static GoudException CreateTyped(
            int code,
            string message,
            string category,
            string subsystem,
            string operation,
            RecoveryClass recovery,
            string hint
        )
        {
            return category switch
            {
                "Context" => new GoudContextException(
                    code, message, subsystem, operation, recovery, hint),
                "Resource" => new GoudResourceException(
                    code, message, subsystem, operation, recovery, hint),
                "Graphics" => new GoudGraphicsException(
                    code, message, subsystem, operation, recovery, hint),
                "Entity" => new GoudEntityException(
                    code, message, subsystem, operation, recovery, hint),
                "Input" => new GoudInputException(
                    code, message, subsystem, operation, recovery, hint),
                "System" => new GoudSystemException(
                    code, message, subsystem, operation, recovery, hint),
                "Provider" => new GoudProviderException(
                    code, message, subsystem, operation, recovery, hint),
                "Internal" => new GoudInternalException(
                    code, message, subsystem, operation, recovery, hint),
                _ => new GoudException(
                    code, message, category, subsystem, operation, recovery, hint),
            };
        }

        // Delegate type matching goud_last_error_message / subsystem / operation
        private delegate int BufferFfiCall(IntPtr buf, nuint bufLen);

        private static string ReadStringFromFfi(BufferFfiCall ffiCall)
        {
            var buf = new byte[256];
            unsafe
            {
                fixed (byte* ptr = buf)
                {
                    int written = ffiCall((IntPtr)ptr, (nuint)buf.Length);
                    if (written <= 0)
                        return string.Empty;
                    return Encoding.UTF8.GetString(buf, 0, written);
                }
            }
        }

        // Delegate for goud_error_recovery_hint (takes code + buffer)
        private static string ReadHintFromFfi(int code)
        {
            var buf = new byte[256];
            unsafe
            {
                fixed (byte* ptr = buf)
                {
                    int written = NativeMethods.goud_error_recovery_hint(
                        code, (IntPtr)ptr, (nuint)buf.Length);
                    if (written <= 0)
                        return string.Empty;
                    return Encoding.UTF8.GetString(buf, 0, written);
                }
            }
        }
    }

    public class GoudContextException : GoudException
    {
        public GoudContextException(
            int code, string message, string subsystem,
            string operation, RecoveryClass recovery, string hint
        ) : base(code, message, "Context", subsystem, operation, recovery, hint) { }
    }

    public class GoudResourceException : GoudException
    {
        public GoudResourceException(
            int code, string message, string subsystem,
            string operation, RecoveryClass recovery, string hint
        ) : base(code, message, "Resource", subsystem, operation, recovery, hint) { }
    }

    public class GoudGraphicsException : GoudException
    {
        public GoudGraphicsException(
            int code, string message, string subsystem,
            string operation, RecoveryClass recovery, string hint
        ) : base(code, message, "Graphics", subsystem, operation, recovery, hint) { }
    }

    public class GoudEntityException : GoudException
    {
        public GoudEntityException(
            int code, string message, string subsystem,
            string operation, RecoveryClass recovery, string hint
        ) : base(code, message, "Entity", subsystem, operation, recovery, hint) { }
    }

    public class GoudInputException : GoudException
    {
        public GoudInputException(
            int code, string message, string subsystem,
            string operation, RecoveryClass recovery, string hint
        ) : base(code, message, "Input", subsystem, operation, recovery, hint) { }
    }

    public class GoudSystemException : GoudException
    {
        public GoudSystemException(
            int code, string message, string subsystem,
            string operation, RecoveryClass recovery, string hint
        ) : base(code, message, "System", subsystem, operation, recovery, hint) { }
    }

    public class GoudProviderException : GoudException
    {
        public GoudProviderException(
            int code, string message, string subsystem,
            string operation, RecoveryClass recovery, string hint
        ) : base(code, message, "Provider", subsystem, operation, recovery, hint) { }
    }

    public class GoudInternalException : GoudException
    {
        public GoudInternalException(
            int code, string message, string subsystem,
            string operation, RecoveryClass recovery, string hint
        ) : base(code, message, "Internal", subsystem, operation, recovery, hint) { }
    }
}

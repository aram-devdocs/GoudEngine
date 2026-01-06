using System;

namespace GoudEngine.Core
{
    /// <summary>
    /// Base exception class for all GoudEngine errors.
    /// Contains the native error code and detailed error information.
    /// </summary>
    public class GoudEngineException : Exception
    {
        /// <summary>
        /// The native error code from the engine.
        /// </summary>
        public int ErrorCode { get; }

        /// <summary>
        /// The error category based on the error code range.
        /// </summary>
        public ErrorCategory Category => GetErrorCategory(ErrorCode);

        /// <summary>
        /// Creates a new GoudEngineException.
        /// </summary>
        /// <param name="errorCode">The native error code.</param>
        /// <param name="message">The error message.</param>
        public GoudEngineException(int errorCode, string message)
            : base(message)
        {
            ErrorCode = errorCode;
        }

        /// <summary>
        /// Creates a new GoudEngineException with an inner exception.
        /// </summary>
        /// <param name="errorCode">The native error code.</param>
        /// <param name="message">The error message.</param>
        /// <param name="innerException">The inner exception.</param>
        public GoudEngineException(int errorCode, string message, Exception innerException)
            : base(message, innerException)
        {
            ErrorCode = errorCode;
        }

        /// <summary>
        /// Determines the error category from an error code.
        /// </summary>
        private static ErrorCategory GetErrorCategory(int errorCode)
        {
            if (errorCode == 0) return ErrorCategory.Success;
            if (errorCode >= 1 && errorCode < 100) return ErrorCategory.Context;
            if (errorCode >= 100 && errorCode < 200) return ErrorCategory.Resource;
            if (errorCode >= 200 && errorCode < 300) return ErrorCategory.Graphics;
            if (errorCode >= 300 && errorCode < 400) return ErrorCategory.Entity;
            if (errorCode >= 400 && errorCode < 500) return ErrorCategory.Input;
            if (errorCode >= 500 && errorCode < 600) return ErrorCategory.System;
            if (errorCode >= 900) return ErrorCategory.Internal;
            return ErrorCategory.Unknown;
        }

        /// <summary>
        /// Returns a string representation of the exception with error code.
        /// </summary>
        public override string ToString()
        {
            return $"[GOUD-{ErrorCode}] {Category}: {Message}";
        }
    }

    /// <summary>
    /// Exception thrown when a context-related operation fails.
    /// Error codes: 1-99
    /// </summary>
    public class ContextException : GoudEngineException
    {
        public ContextException(int errorCode, string message)
            : base(errorCode, message) { }

        public ContextException(int errorCode, string message, Exception innerException)
            : base(errorCode, message, innerException) { }
    }

    /// <summary>
    /// Exception thrown when a resource or asset operation fails.
    /// Error codes: 100-199
    /// </summary>
    public class ResourceException : GoudEngineException
    {
        /// <summary>
        /// The path to the resource that failed to load, if applicable.
        /// </summary>
        public string? ResourcePath { get; set; }

        public ResourceException(int errorCode, string message)
            : base(errorCode, message) { }

        public ResourceException(int errorCode, string message, string? resourcePath)
            : base(errorCode, message)
        {
            ResourcePath = resourcePath;
        }

        public ResourceException(int errorCode, string message, Exception innerException)
            : base(errorCode, message, innerException) { }

        public override string ToString()
        {
            var baseString = base.ToString();
            return ResourcePath != null
                ? $"{baseString} (Resource: {ResourcePath})"
                : baseString;
        }
    }

    /// <summary>
    /// Exception thrown when a graphics operation fails.
    /// Error codes: 200-299
    /// </summary>
    public class GraphicsException : GoudEngineException
    {
        public GraphicsException(int errorCode, string message)
            : base(errorCode, message) { }

        public GraphicsException(int errorCode, string message, Exception innerException)
            : base(errorCode, message, innerException) { }
    }

    /// <summary>
    /// Exception thrown when an entity operation fails.
    /// Error codes: 300-399
    /// </summary>
    public class EntityException : GoudEngineException
    {
        /// <summary>
        /// The entity ID involved in the failed operation, if applicable.
        /// </summary>
        public GoudEntityId? EntityId { get; set; }

        public EntityException(int errorCode, string message)
            : base(errorCode, message) { }

        public EntityException(int errorCode, string message, GoudEntityId? entityId)
            : base(errorCode, message)
        {
            EntityId = entityId;
        }

        public EntityException(int errorCode, string message, Exception innerException)
            : base(errorCode, message, innerException) { }

        public override string ToString()
        {
            var baseString = base.ToString();
            return EntityId.HasValue
                ? $"{baseString} (Entity: {EntityId.Value})"
                : baseString;
        }
    }

    /// <summary>
    /// Exception thrown when an input operation fails.
    /// Error codes: 400-499
    /// </summary>
    public class InputException : GoudEngineException
    {
        public InputException(int errorCode, string message)
            : base(errorCode, message) { }

        public InputException(int errorCode, string message, Exception innerException)
            : base(errorCode, message, innerException) { }
    }

    /// <summary>
    /// Exception thrown when a system or platform operation fails.
    /// Error codes: 500-599
    /// </summary>
    public class SystemException : GoudEngineException
    {
        public SystemException(int errorCode, string message)
            : base(errorCode, message) { }

        public SystemException(int errorCode, string message, Exception innerException)
            : base(errorCode, message, innerException) { }
    }

    /// <summary>
    /// Exception thrown when an internal engine error occurs.
    /// Error codes: 900-999
    /// </summary>
    public class InternalException : GoudEngineException
    {
        public InternalException(int errorCode, string message)
            : base(errorCode, message) { }

        public InternalException(int errorCode, string message, Exception innerException)
            : base(errorCode, message, innerException) { }
    }

    /// <summary>
    /// Error category enumeration matching the Rust error code ranges.
    /// </summary>
    public enum ErrorCategory
    {
        /// <summary>Success (error code 0)</summary>
        Success = 0,
        /// <summary>Context/initialization errors (1-99)</summary>
        Context = 1,
        /// <summary>Resource/asset errors (100-199)</summary>
        Resource = 100,
        /// <summary>Graphics/rendering errors (200-299)</summary>
        Graphics = 200,
        /// <summary>Entity/ECS errors (300-399)</summary>
        Entity = 300,
        /// <summary>Input handling errors (400-499)</summary>
        Input = 400,
        /// <summary>System/platform errors (500-599)</summary>
        System = 500,
        /// <summary>Internal/unexpected errors (900-999)</summary>
        Internal = 900,
        /// <summary>Unknown error category</summary>
        Unknown = -1
    }

    /// <summary>
    /// Helper class for creating exceptions from FFI error codes.
    /// </summary>
    public static class ErrorHelper
    {
        /// <summary>
        /// Creates the appropriate exception type based on the error code.
        /// </summary>
        /// <param name="errorCode">The native error code.</param>
        /// <param name="message">The error message.</param>
        /// <returns>A specific exception type based on the error category.</returns>
        public static GoudEngineException CreateException(int errorCode, string message)
        {
            if (errorCode >= 1 && errorCode < 100)
                return new ContextException(errorCode, message);
            if (errorCode >= 100 && errorCode < 200)
                return new ResourceException(errorCode, message);
            if (errorCode >= 200 && errorCode < 300)
                return new GraphicsException(errorCode, message);
            if (errorCode >= 300 && errorCode < 400)
                return new EntityException(errorCode, message);
            if (errorCode >= 400 && errorCode < 500)
                return new InputException(errorCode, message);
            if (errorCode >= 500 && errorCode < 600)
                return new SystemException(errorCode, message);
            if (errorCode >= 900)
                return new InternalException(errorCode, message);

            return new GoudEngineException(errorCode, message);
        }

        /// <summary>
        /// Checks a result code and throws an exception if it indicates failure.
        /// </summary>
        /// <param name="success">Whether the operation succeeded.</param>
        /// <param name="errorCode">The error code if failed.</param>
        /// <param name="operation">Description of the operation that was attempted.</param>
        /// <exception cref="GoudEngineException">Thrown if result indicates failure.</exception>
        public static void ThrowIfFailed(bool success, int errorCode, string operation)
        {
            if (!success)
            {
                var message = $"{operation} failed with error code {errorCode}";
                throw CreateException(errorCode, message);
            }
        }

        /// <summary>
        /// Validates an entity ID and throws if invalid.
        /// </summary>
        /// <param name="entityId">The entity ID to validate.</param>
        /// <param name="paramName">The parameter name for the exception.</param>
        /// <exception cref="ArgumentException">Thrown if entity ID is invalid.</exception>
        public static void ValidateEntityId(GoudEntityId entityId, string paramName)
        {
            if (entityId.IsInvalid)
            {
                throw new ArgumentException("Entity ID is invalid", paramName);
            }
        }

        /// <summary>
        /// Validates a context ID and throws if invalid.
        /// </summary>
        /// <param name="contextId">The context ID to validate.</param>
        /// <param name="paramName">The parameter name for the exception.</param>
        /// <exception cref="ArgumentException">Thrown if context ID is invalid.</exception>
        public static void ValidateContextId(GoudContextId contextId, string paramName)
        {
            if (contextId.IsInvalid)
            {
                throw new ArgumentException("Context ID is invalid", paramName);
            }
        }

        /// <summary>
        /// Validates a handle and throws if invalid.
        /// </summary>
        /// <param name="handle">The handle value to validate.</param>
        /// <param name="paramName">The parameter name for the exception.</param>
        /// <exception cref="ArgumentException">Thrown if handle is invalid.</exception>
        public static void ValidateHandle(ulong handle, string paramName)
        {
            if (handle == 0 || handle == ulong.MaxValue)
            {
                throw new ArgumentException("Handle is invalid", paramName);
            }
        }
    }

    /// <summary>
    /// Extension methods for safe error handling.
    /// </summary>
    public static class ErrorExtensions
    {
        /// <summary>
        /// Attempts an operation and returns a result indicating success or failure.
        /// </summary>
        /// <typeparam name="T">The result type.</typeparam>
        /// <param name="operation">The operation to attempt.</param>
        /// <param name="result">The result value if successful.</param>
        /// <param name="error">The exception if failed.</param>
        /// <returns>True if successful, false otherwise.</returns>
        public static bool TryExecute<T>(Func<T> operation, out T? result, out Exception? error)
        {
            try
            {
                result = operation();
                error = null;
                return true;
            }
            catch (Exception ex)
            {
                result = default;
                error = ex;
                return false;
            }
        }

        /// <summary>
        /// Attempts an operation and returns a result indicating success or failure.
        /// </summary>
        /// <param name="operation">The operation to attempt.</param>
        /// <param name="error">The exception if failed.</param>
        /// <returns>True if successful, false otherwise.</returns>
        public static bool TryExecute(Action operation, out Exception? error)
        {
            try
            {
                operation();
                error = null;
                return true;
            }
            catch (Exception ex)
            {
                error = ex;
                return false;
            }
        }
    }
}

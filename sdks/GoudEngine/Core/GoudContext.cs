using System;
using System.Runtime.InteropServices;
using CsBindgen;

namespace GoudEngine.Core
{
    /// <summary>
    /// Represents a GoudEngine context ID returned from native code.
    /// This is an opaque 64-bit identifier with generational indexing.
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct GoudContextId : IEquatable<GoudContextId>
    {
        private readonly ulong value;

        internal GoudContextId(ulong value)
        {
            this.value = value;
        }

        /// <summary>
        /// The invalid context ID sentinel value (all bits set).
        /// </summary>
        public static readonly GoudContextId Invalid = new GoudContextId(ulong.MaxValue);

        /// <summary>
        /// Returns true if this is an invalid context ID.
        /// </summary>
        public bool IsInvalid => value == ulong.MaxValue;

        /// <summary>
        /// Returns true if this is a valid context ID.
        /// </summary>
        public bool IsValid => !IsInvalid;

        /// <summary>
        /// Returns the index component (lower 32 bits).
        /// </summary>
        public uint Index => (uint)(value & 0xFFFFFFFF);

        /// <summary>
        /// Returns the generation component (upper 32 bits).
        /// </summary>
        public uint Generation => (uint)(value >> 32);

        /// <summary>
        /// Implicit conversion to ulong for FFI calls.
        /// </summary>
        public static implicit operator ulong(GoudContextId id) => id.value;

        /// <summary>
        /// Implicit conversion from ulong for FFI returns.
        /// </summary>
        public static implicit operator GoudContextId(ulong value) => new GoudContextId(value);

        public bool Equals(GoudContextId other) => value == other.value;
        public override bool Equals(object? obj) => obj is GoudContextId other && Equals(other);
        public override int GetHashCode() => value.GetHashCode();
        public static bool operator ==(GoudContextId left, GoudContextId right) => left.Equals(right);
        public static bool operator !=(GoudContextId left, GoudContextId right) => !left.Equals(right);

        public override string ToString()
        {
            return IsInvalid ? "GoudContextId(INVALID)" : $"GoudContextId({Index}:{Generation})";
        }
    }

    /// <summary>
    /// Represents a GoudEngine entity ID.
    /// This is an opaque 64-bit identifier with generational indexing.
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct GoudEntityId : IEquatable<GoudEntityId>
    {
        private readonly ulong value;

        internal GoudEntityId(ulong value)
        {
            this.value = value;
        }

        /// <summary>
        /// The invalid entity ID sentinel value (all bits set).
        /// </summary>
        public static readonly GoudEntityId Invalid = new GoudEntityId(ulong.MaxValue);

        /// <summary>
        /// Returns true if this is an invalid entity ID.
        /// </summary>
        public bool IsInvalid => value == ulong.MaxValue;

        /// <summary>
        /// Returns true if this is a valid entity ID.
        /// </summary>
        public bool IsValid => !IsInvalid;

        /// <summary>
        /// Implicit conversion to ulong for FFI calls.
        /// </summary>
        public static implicit operator ulong(GoudEntityId id) => id.value;

        /// <summary>
        /// Implicit conversion from ulong for FFI returns.
        /// </summary>
        public static implicit operator GoudEntityId(ulong value) => new GoudEntityId(value);

        public bool Equals(GoudEntityId other) => value == other.value;
        public override bool Equals(object? obj) => obj is GoudEntityId other && Equals(other);
        public override int GetHashCode() => value.GetHashCode();
        public static bool operator ==(GoudEntityId left, GoudEntityId right) => left.Equals(right);
        public static bool operator !=(GoudEntityId left, GoudEntityId right) => !left.Equals(right);

        public override string ToString()
        {
            return IsInvalid ? "GoudEntityId(INVALID)" : $"GoudEntityId({value})";
        }
    }

    /// <summary>
    /// Represents a GoudEngine context - a single engine instance with its own World,
    /// entities, components, resources, and assets. Multiple contexts can exist
    /// simultaneously for multiple game instances or editor viewports.
    /// </summary>
    public sealed class GoudContext : IDisposable
    {
        private GoudContextId _contextId;
        private bool _disposed;

        /// <summary>
        /// Gets the native context ID for this context.
        /// </summary>
        public GoudContextId ContextId => _contextId;

        /// <summary>
        /// Returns true if this context has been disposed.
        /// </summary>
        public bool IsDisposed => _disposed;

        /// <summary>
        /// Returns true if this context is valid (created and not destroyed).
        /// </summary>
        public bool IsValid => !_disposed && _contextId.IsValid && NativeMethods.goud_context_is_valid(_contextId);

        /// <summary>
        /// Creates a new GoudEngine context.
        /// </summary>
        /// <exception cref="GoudEngineException">Thrown if context creation fails.</exception>
        public GoudContext()
        {
            _contextId = NativeMethods.goud_context_create();

            if (_contextId.IsInvalid)
            {
                throw new GoudEngineException(
                    -1,
                    "Failed to create GoudEngine context"
                );
            }
        }

        /// <summary>
        /// Internal constructor for wrapping an existing context ID.
        /// </summary>
        internal GoudContext(GoudContextId contextId)
        {
            _contextId = contextId;

            if (_contextId.IsInvalid)
            {
                throw new ArgumentException("Cannot wrap invalid context ID", nameof(contextId));
            }
        }

        /// <summary>
        /// Validates that this context is valid and not disposed.
        /// </summary>
        /// <exception cref="ObjectDisposedException">Thrown if context is disposed.</exception>
        /// <exception cref="InvalidOperationException">Thrown if context is invalid.</exception>
        private void ThrowIfInvalid()
        {
            if (_disposed)
            {
                throw new ObjectDisposedException(nameof(GoudContext));
            }

            if (!_contextId.IsValid)
            {
                throw new InvalidOperationException("Context ID is invalid");
            }

            if (!NativeMethods.goud_context_is_valid(_contextId))
            {
                throw new InvalidOperationException("Context has been destroyed on the native side");
            }
        }

        /// <summary>
        /// Gets the last error message from the engine.
        /// </summary>
        /// <returns>The error message, or a generic message if unavailable.</returns>
        private static string GetLastErrorMessage()
        {
            // TODO: Implement goud_last_error_message() FFI function in Phase 6
            // For now, return a generic message
            return "An error occurred in the engine";
        }

        /// <summary>
        /// Gets the last error code from the engine.
        /// </summary>
        /// <returns>The error code (0 = success, non-zero = error).</returns>
        public static int GetLastErrorCode()
        {
            // TODO: Implement goud_last_error_code() FFI function in Phase 6
            // For now, return a generic error code
            return -1;
        }

        /// <summary>
        /// Clears the last error stored in thread-local storage.
        /// </summary>
        public static void ClearLastError()
        {
            // TODO: Implement goud_clear_last_error() FFI function in Phase 6
            // No-op for now
        }

        // ====================================================================
        // Entity Operations
        // ====================================================================

        /// <summary>
        /// Spawns a new empty entity in this context.
        /// </summary>
        /// <returns>The ID of the spawned entity.</returns>
        /// <exception cref="GoudEngineException">Thrown if entity spawning fails.</exception>
        public GoudEntityId SpawnEntity()
        {
            ThrowIfInvalid();

            var entityId = NativeMethods.goud_entity_spawn_empty(_contextId);

            if (entityId == GoudEntityId.Invalid)
            {
                throw new GoudEngineException(
                    -1,
                    "Failed to spawn entity"
                );
            }

            return entityId;
        }

        /// <summary>
        /// Spawns multiple empty entities in a batch.
        /// </summary>
        /// <param name="count">The number of entities to spawn.</param>
        /// <returns>An array of spawned entity IDs.</returns>
        /// <exception cref="ArgumentOutOfRangeException">Thrown if count is negative or zero.</exception>
        /// <exception cref="GoudEngineException">Thrown if batch spawning fails.</exception>
        public unsafe GoudEntityId[] SpawnEntitiesBatch(int count)
        {
            if (count <= 0)
            {
                throw new ArgumentOutOfRangeException(nameof(count), "Count must be positive");
            }

            ThrowIfInvalid();

            var entities = new ulong[count];
            fixed (ulong* entitiesPtr = entities)
            {
                uint spawned = NativeMethods.goud_entity_spawn_batch(_contextId, (uint)count, entitiesPtr);

                if (spawned != (uint)count)
                {
                    throw new GoudEngineException(
                        -1,
                        $"Failed to spawn all entities: spawned {spawned} of {count}"
                    );
                }
            }

            // Convert ulong array to GoudEntityId array
            var result = new GoudEntityId[count];
            for (int i = 0; i < count; i++)
            {
                result[i] = new GoudEntityId(entities[i]);
            }
            return result;
        }

        /// <summary>
        /// Despawns an entity, removing it and all its components from the world.
        /// </summary>
        /// <param name="entityId">The entity to despawn.</param>
        /// <returns>True if the entity was despawned, false if it didn't exist.</returns>
        public bool DespawnEntity(GoudEntityId entityId)
        {
            ThrowIfInvalid();

            var result = NativeMethods.goud_entity_despawn(_contextId, entityId);
            return result.Success;
        }

        /// <summary>
        /// Despawns multiple entities in a batch.
        /// </summary>
        /// <param name="entityIds">The entities to despawn.</param>
        /// <returns>The number of entities successfully despawned.</returns>
        public unsafe uint DespawnEntitiesBatch(GoudEntityId[] entityIds)
        {
            if (entityIds == null || entityIds.Length == 0)
            {
                return 0;
            }

            ThrowIfInvalid();

            // Convert GoudEntityId array to ulong array
            var entities = new ulong[entityIds.Length];
            for (int i = 0; i < entityIds.Length; i++)
            {
                entities[i] = entityIds[i];
            }

            fixed (ulong* entitiesPtr = entities)
            {
                return NativeMethods.goud_entity_despawn_batch(_contextId, entitiesPtr, (uint)entityIds.Length);
            }
        }

        /// <summary>
        /// Checks if an entity is alive in this context.
        /// </summary>
        /// <param name="entityId">The entity to check.</param>
        /// <returns>True if the entity is alive, false otherwise.</returns>
        public bool IsEntityAlive(GoudEntityId entityId)
        {
            ThrowIfInvalid();

            return NativeMethods.goud_entity_is_alive(_contextId, entityId);
        }

        /// <summary>
        /// Checks if multiple entities are alive in a batch.
        /// </summary>
        /// <param name="entityIds">The entities to check.</param>
        /// <returns>An array of booleans indicating which entities are alive.</returns>
        public unsafe bool[] IsEntitiesAliveBatch(GoudEntityId[] entityIds)
        {
            if (entityIds == null || entityIds.Length == 0)
            {
                return Array.Empty<bool>();
            }

            ThrowIfInvalid();

            // Convert GoudEntityId array to ulong array
            var entities = new ulong[entityIds.Length];
            for (int i = 0; i < entityIds.Length; i++)
            {
                entities[i] = entityIds[i];
            }

            // Native expects byte* for boolean results
            var resultsBytes = new byte[entityIds.Length];
            fixed (ulong* entitiesPtr = entities)
            fixed (byte* resultsPtr = resultsBytes)
            {
                NativeMethods.goud_entity_is_alive_batch(_contextId, entitiesPtr, (uint)entityIds.Length, resultsPtr);
            }

            // Convert bytes to bools
            var results = new bool[entityIds.Length];
            for (int i = 0; i < entityIds.Length; i++)
            {
                results[i] = resultsBytes[i] != 0;
            }

            return results;
        }

        /// <summary>
        /// Gets the total number of entities in this context.
        /// </summary>
        public uint EntityCount
        {
            get
            {
                ThrowIfInvalid();
                return NativeMethods.goud_entity_count(_contextId);
            }
        }

        // ====================================================================
        // IDisposable Implementation
        // ====================================================================

        /// <summary>
        /// Finalizer to ensure native resources are freed if Dispose is not called.
        /// </summary>
        ~GoudContext()
        {
            Dispose(false);
        }

        /// <summary>
        /// Disposes the context, freeing all native resources.
        /// </summary>
        public void Dispose()
        {
            Dispose(true);
            GC.SuppressFinalize(this);
        }

        /// <summary>
        /// Disposes the context.
        /// </summary>
        /// <param name="disposing">True if called from Dispose(), false if called from finalizer.</param>
        private void Dispose(bool disposing)
        {
            if (_disposed)
            {
                return;
            }

            // Free native resources
            if (_contextId.IsValid)
            {
                bool success = NativeMethods.goud_context_destroy(_contextId);

                // In finalizer, don't throw exceptions
                if (!success && disposing)
                {
                    System.Diagnostics.Debug.WriteLine(
                        $"Warning: Failed to destroy context {_contextId}"
                    );
                }

                _contextId = GoudContextId.Invalid;
            }

            _disposed = true;
        }

        public override string ToString()
        {
            return _disposed ? "GoudContext(Disposed)" : $"GoudContext({_contextId})";
        }
    }
}

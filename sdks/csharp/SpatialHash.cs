using System;
using System.Runtime.InteropServices;

namespace GoudEngine
{
    /// <summary>
    /// AABB-based spatial hash for broad-phase collision queries.
    /// Wraps the native Rust <c>SpatialHash</c> via FFI.
    /// </summary>
    public class SpatialHash : IDisposable
    {
        private uint _handle;
        private bool _disposed;

        /// <summary>
        /// Creates a new spatial hash with the specified cell size.
        /// </summary>
        /// <param name="cellSize">Size of each grid cell in world units. Must be positive.</param>
        public SpatialHash(float cellSize)
        {
            _handle = NativeMethods.goud_spatial_hash_create(cellSize);
            if (_handle == uint.MaxValue)
                throw new InvalidOperationException("Failed to create SpatialHash");
        }

        /// <summary>
        /// Creates a new spatial hash with pre-allocated capacity.
        /// </summary>
        /// <param name="cellSize">Size of each grid cell in world units. Must be positive.</param>
        /// <param name="capacity">Expected number of entities.</param>
        public SpatialHash(float cellSize, uint capacity)
        {
            _handle = NativeMethods.goud_spatial_hash_create_with_capacity(cellSize, capacity);
            if (_handle == uint.MaxValue)
                throw new InvalidOperationException("Failed to create SpatialHash");
        }

        /// <summary>
        /// Inserts an entity with an AABB into the spatial hash.
        /// If the entity already exists, its AABB is overwritten.
        /// </summary>
        /// <param name="entityId">Entity ID.</param>
        /// <param name="x">Center X position.</param>
        /// <param name="y">Center Y position.</param>
        /// <param name="halfW">Half-width of the AABB.</param>
        /// <param name="halfH">Half-height of the AABB.</param>
        public void Insert(ulong entityId, float x, float y, float halfW, float halfH)
        {
            ThrowIfDisposed();
            int result = NativeMethods.goud_spatial_hash_insert(_handle, entityId, x, y, halfW, halfH);
            if (result != 0)
                throw new InvalidOperationException("SpatialHash.Insert failed");
        }

        /// <summary>
        /// Removes an entity from the spatial hash.
        /// </summary>
        /// <param name="entityId">Entity ID.</param>
        public void Remove(ulong entityId)
        {
            ThrowIfDisposed();
            int result = NativeMethods.goud_spatial_hash_remove(_handle, entityId);
            if (result != 0)
                throw new InvalidOperationException("SpatialHash.Remove failed");
        }

        /// <summary>
        /// Updates an entity's AABB in the spatial hash.
        /// The entity must already exist.
        /// </summary>
        /// <param name="entityId">Entity ID.</param>
        /// <param name="x">New center X position.</param>
        /// <param name="y">New center Y position.</param>
        /// <param name="halfW">New half-width of the AABB.</param>
        /// <param name="halfH">New half-height of the AABB.</param>
        public void Update(ulong entityId, float x, float y, float halfW, float halfH)
        {
            ThrowIfDisposed();
            int result = NativeMethods.goud_spatial_hash_update(_handle, entityId, x, y, halfW, halfH);
            if (result != 0)
                throw new InvalidOperationException("SpatialHash.Update failed");
        }

        /// <summary>
        /// Clears all entities from the spatial hash.
        /// </summary>
        public void Clear()
        {
            ThrowIfDisposed();
            int result = NativeMethods.goud_spatial_hash_clear(_handle);
            if (result != 0)
                throw new InvalidOperationException("SpatialHash.Clear failed");
        }

        /// <summary>
        /// Queries for entities within a radius of a center point.
        /// </summary>
        /// <param name="x">Center X position.</param>
        /// <param name="y">Center Y position.</param>
        /// <param name="radius">Query radius.</param>
        /// <param name="results">Buffer to receive entity IDs.</param>
        /// <returns>Total number of entities found (may exceed buffer size).</returns>
        public int QueryRange(float x, float y, float radius, Span<ulong> results)
        {
            ThrowIfDisposed();
            if (results.Length == 0)
            {
                ulong dummy = 0;
                return NativeMethods.goud_spatial_hash_query_range(
                    _handle, x, y, radius, ref dummy, 0);
            }
            return NativeMethods.goud_spatial_hash_query_range(
                _handle, x, y, radius,
                ref MemoryMarshal.GetReference(results),
                (uint)results.Length);
        }

        /// <summary>
        /// Queries for entities whose AABB overlaps the given rectangle.
        /// </summary>
        /// <param name="x">Rectangle X position (top-left).</param>
        /// <param name="y">Rectangle Y position (top-left).</param>
        /// <param name="w">Rectangle width.</param>
        /// <param name="h">Rectangle height.</param>
        /// <param name="results">Buffer to receive entity IDs.</param>
        /// <returns>Total number of entities found (may exceed buffer size).</returns>
        public int QueryRect(float x, float y, float w, float h, Span<ulong> results)
        {
            ThrowIfDisposed();
            if (results.Length == 0)
            {
                ulong dummy = 0;
                return NativeMethods.goud_spatial_hash_query_rect(
                    _handle, x, y, w, h, ref dummy, 0);
            }
            return NativeMethods.goud_spatial_hash_query_rect(
                _handle, x, y, w, h,
                ref MemoryMarshal.GetReference(results),
                (uint)results.Length);
        }

        /// <summary>
        /// Gets the number of entities in the spatial hash.
        /// </summary>
        public int Count
        {
            get
            {
                ThrowIfDisposed();
                return NativeMethods.goud_spatial_hash_entity_count(_handle);
            }
        }

        /// <summary>
        /// Disposes the spatial hash and frees native resources.
        /// </summary>
        public void Dispose()
        {
            if (!_disposed)
            {
                NativeMethods.goud_spatial_hash_destroy(_handle);
                _handle = uint.MaxValue;
                _disposed = true;
            }
        }

        private void ThrowIfDisposed()
        {
            if (_disposed)
                throw new ObjectDisposedException(nameof(SpatialHash));
        }
    }
}

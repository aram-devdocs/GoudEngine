using System;
using System.Runtime.InteropServices;
using System.Text;
using CsBindgen;
using GoudEngine.Components;

namespace GoudEngine.Core
{
    /// <summary>
    /// Represents an entity in the GoudEngine ECS system.
    /// Entities are lightweight identifiers that can have components attached.
    /// This class provides a safe, object-oriented wrapper around the raw entity ID.
    /// </summary>
    public sealed class Entity : IEquatable<Entity>
    {
        private readonly GoudContext _context;
        private readonly GoudEntityId _entityId;

        /// <summary>
        /// Gets the raw entity ID for this entity.
        /// </summary>
        public GoudEntityId Id => _entityId;

        /// <summary>
        /// Gets the context this entity belongs to.
        /// </summary>
        public GoudContext Context => _context;

        /// <summary>
        /// Returns true if this entity is currently alive in the world.
        /// </summary>
        public bool IsAlive => _context.IsEntityAlive(_entityId);

        /// <summary>
        /// Returns true if this entity has been despawned or is invalid.
        /// </summary>
        public bool IsDead => !IsAlive;

        /// <summary>
        /// Internal constructor for creating an Entity wrapper.
        /// Use <see cref="GoudContext.SpawnEntity()"/> to create entities.
        /// </summary>
        /// <param name="context">The context this entity belongs to.</param>
        /// <param name="entityId">The raw entity ID.</param>
        internal Entity(GoudContext context, GoudEntityId entityId)
        {
            _context = context ?? throw new ArgumentNullException(nameof(context));
            _entityId = entityId;

            if (entityId.Item1 == ulong.MaxValue)
            {
                throw new ArgumentException("Cannot create Entity with invalid ID", nameof(entityId));
            }
        }

        /// <summary>
        /// Validates that this entity is alive and its context is valid.
        /// </summary>
        /// <exception cref="ObjectDisposedException">Thrown if context is disposed.</exception>
        /// <exception cref="InvalidOperationException">Thrown if entity is dead.</exception>
        private void ThrowIfDead()
        {
            if (_context.IsDisposed)
            {
                throw new ObjectDisposedException(nameof(GoudContext), "Entity's context has been disposed");
            }

            if (!IsAlive)
            {
                throw new InvalidOperationException($"Entity {_entityId.Item1} is no longer alive");
            }
        }

        // ====================================================================
        // Lifecycle Methods
        // ====================================================================

        /// <summary>
        /// Despawns this entity, removing it and all its components from the world.
        /// After calling this method, the entity will be dead and unusable.
        /// </summary>
        /// <returns>True if the entity was despawned, false if it was already dead.</returns>
        public bool Despawn()
        {
            if (_context.IsDisposed || _entityId.Item1 == ulong.MaxValue)
            {
                return false;
            }

            return _context.DespawnEntity(_entityId);
        }

        // ====================================================================
        // Component Operations
        // ====================================================================

        /// <summary>
        /// Adds a component to this entity.
        /// </summary>
        /// <typeparam name="T">The component type to add.</typeparam>
        /// <param name="component">The component data.</param>
        /// <returns>This entity for method chaining.</returns>
        /// <exception cref="InvalidOperationException">Thrown if entity is dead or component lacks attribute.</exception>
        public Entity AddComponent<T>(T component) where T : struct, IComponent
        {
            ThrowIfDead();

            // Get component metadata
            var attribute = GetComponentAttribute<T>();
            if (attribute == null)
            {
                throw new InvalidOperationException($"Component type {typeof(T).Name} lacks [Component] attribute");
            }

            // Register type with Rust if not already done
            EnsureTypeRegistered<T>(attribute);

            unsafe
            {
                // Marshal component to unmanaged memory
                var size = Marshal.SizeOf<T>();
                var ptr = Marshal.AllocHGlobal(size);
                try
                {
                    Marshal.StructureToPtr(component, ptr, false);

                    var result = NativeMethods.goud_component_add(
                        _context.ContextId,
                        _entityId,
                        attribute.TypeId,
                        (byte*)ptr.ToPointer(),
                        (nuint)size
                    );

                    if (!result.Success)
                    {
                        throw new InvalidOperationException(
                            $"Failed to add component {typeof(T).Name} to entity {_entityId.Item1}. Error code: {result.code}");
                    }
                }
                finally
                {
                    Marshal.FreeHGlobal(ptr);
                }
            }

            return this;
        }

        /// <summary>
        /// Removes a component from this entity.
        /// </summary>
        /// <typeparam name="T">The component type to remove.</typeparam>
        /// <returns>True if the component was removed, false if entity didn't have it.</returns>
        /// <exception cref="InvalidOperationException">Thrown if entity is dead or component lacks attribute.</exception>
        public bool RemoveComponent<T>() where T : struct, IComponent
        {
            ThrowIfDead();

            // Get component metadata
            var attribute = GetComponentAttribute<T>();
            if (attribute == null)
            {
                throw new InvalidOperationException($"Component type {typeof(T).Name} lacks [Component] attribute");
            }

            var result = NativeMethods.goud_component_remove(
                _context.ContextId,
                _entityId,
                attribute.TypeId
            );

            return result.Success;
        }

        /// <summary>
        /// Checks if this entity has a component of the specified type.
        /// </summary>
        /// <typeparam name="T">The component type to check.</typeparam>
        /// <returns>True if the entity has the component, false otherwise.</returns>
        /// <exception cref="InvalidOperationException">Thrown if entity is dead or component lacks attribute.</exception>
        public bool HasComponent<T>() where T : struct, IComponent
        {
            ThrowIfDead();

            // Get component metadata
            var attribute = GetComponentAttribute<T>();
            if (attribute == null)
            {
                throw new InvalidOperationException($"Component type {typeof(T).Name} lacks [Component] attribute");
            }

            return NativeMethods.goud_component_has(
                _context.ContextId,
                _entityId,
                attribute.TypeId
            );
        }

        /// <summary>
        /// Gets a copy of a component on this entity.
        /// Note: Returns a copy, not a reference.
        /// </summary>
        /// <typeparam name="T">The component type to get.</typeparam>
        /// <returns>A copy of the component data.</returns>
        /// <exception cref="InvalidOperationException">Thrown if entity is dead or doesn't have component.</exception>
        public T GetComponent<T>() where T : struct, IComponent
        {
            ThrowIfDead();

            // Get component metadata
            var attribute = GetComponentAttribute<T>();
            if (attribute == null)
            {
                throw new InvalidOperationException($"Component type {typeof(T).Name} lacks [Component] attribute");
            }

            unsafe
            {
                var ptr = NativeMethods.goud_component_get(
                    _context.ContextId,
                    _entityId,
                    attribute.TypeId
                );

                if (ptr == null)
                {
                    throw new InvalidOperationException(
                        $"Entity {_entityId.Item1} does not have component {typeof(T).Name}");
                }

                // Marshal from unmanaged to managed
                return Marshal.PtrToStructure<T>(new IntPtr(ptr));
            }
        }

        /// <summary>
        /// Tries to get a component on this entity.
        /// </summary>
        /// <typeparam name="T">The component type to get.</typeparam>
        /// <param name="component">Output parameter for the component data.</param>
        /// <returns>True if the component exists, false otherwise.</returns>
        public bool TryGetComponent<T>(out T component) where T : struct, IComponent
        {
            component = default;

            if (_context.IsDisposed || !IsAlive)
            {
                return false;
            }

            // Get component metadata
            var attribute = GetComponentAttribute<T>();
            if (attribute == null)
            {
                return false;
            }

            unsafe
            {
                var ptr = NativeMethods.goud_component_get(
                    _context.ContextId,
                    _entityId,
                    attribute.TypeId
                );

                if (ptr == null)
                {
                    return false;
                }

                // Marshal from unmanaged to managed
                component = Marshal.PtrToStructure<T>(new IntPtr(ptr));
                return true;
            }
        }

        /// <summary>
        /// Updates a component on this entity.
        /// This removes the old component and adds the new one.
        /// </summary>
        /// <typeparam name="T">The component type to update.</typeparam>
        /// <param name="component">The new component data.</param>
        /// <returns>This entity for method chaining.</returns>
        /// <exception cref="InvalidOperationException">Thrown if entity is dead.</exception>
        public Entity UpdateComponent<T>(T component) where T : struct, IComponent
        {
            ThrowIfDead();

            // Remove old component (if exists) and add new one
            // Note: This is not atomic in the FFI layer yet
            RemoveComponent<T>();
            AddComponent(component);

            return this;
        }

        // ====================================================================
        // Helper Methods
        // ====================================================================

        /// <summary>
        /// Gets the ComponentAttribute for a type, or null if not present.
        /// </summary>
        private static ComponentAttribute? GetComponentAttribute<T>() where T : struct, IComponent
        {
            var componentType = typeof(T);
            var attributes = componentType.GetCustomAttributes(typeof(ComponentAttribute), false);

            if (attributes.Length == 0)
            {
                return null;
            }

            return (ComponentAttribute)attributes[0];
        }

        /// <summary>
        /// Ensures the component type is registered with the Rust engine.
        /// </summary>
        private static void EnsureTypeRegistered<T>(ComponentAttribute attribute) where T : struct, IComponent
        {
            var name = typeof(T).Name;
            var size = Marshal.SizeOf<T>();
            // Alignment must be a power of 2. For structs with floats, use 4-byte alignment.
            // Calculate the natural alignment (largest power of 2 that divides evenly into size, capped at 8)
            var align = CalculateAlignment(size);

            unsafe
            {
                var nameBytes = Encoding.UTF8.GetBytes(name);
                fixed (byte* namePtr = nameBytes)
                {
                    // This is idempotent - safe to call multiple times
                    NativeMethods.goud_component_register_type(
                        attribute.TypeId,
                        namePtr,
                        (nuint)nameBytes.Length,
                        (nuint)size,
                        (nuint)align
                    );
                }
            }
        }

        /// <summary>
        /// Calculates the appropriate memory alignment for a struct.
        /// Alignment must be a power of 2. For typical game components containing floats,
        /// 4-byte alignment is usually correct.
        /// </summary>
        private static int CalculateAlignment(int size)
        {
            // For most component structs with float fields, 4-byte alignment is correct
            // Larger types might need 8-byte alignment for double/long fields
            if (size == 0) return 1;
            if (size >= 8 && size % 8 == 0) return 8;
            if (size >= 4 && size % 4 == 0) return 4;
            if (size >= 2 && size % 2 == 0) return 2;
            return 1;
        }

        // ====================================================================
        // Equality and Hashing
        // ====================================================================

        /// <summary>
        /// Checks if this entity equals another entity.
        /// Two entities are equal if they have the same ID and belong to the same context.
        /// </summary>
        public bool Equals(Entity? other)
        {
            if (other is null) return false;
            if (ReferenceEquals(this, other)) return true;
            return _entityId.Item1 == other._entityId.Item1 && ReferenceEquals(_context, other._context);
        }

        public override bool Equals(object? obj)
        {
            return obj is Entity other && Equals(other);
        }

        public override int GetHashCode()
        {
            return HashCode.Combine(_entityId.Item1, _context);
        }

        public static bool operator ==(Entity? left, Entity? right)
        {
            if (left is null) return right is null;
            return left.Equals(right);
        }

        public static bool operator !=(Entity? left, Entity? right)
        {
            return !(left == right);
        }

        // ====================================================================
        // String Representation
        // ====================================================================

        public override string ToString()
        {
            if (_entityId.Item1 == ulong.MaxValue)
            {
                return "Entity(INVALID)";
            }

            string status = IsAlive ? "Alive" : "Dead";
            return $"Entity({_entityId.Item1}, {status})";
        }
    }

    /// <summary>
    /// Extension methods for GoudContext to work with Entity wrappers.
    /// </summary>
    public static class EntityContextExtensions
    {
        /// <summary>
        /// Spawns a new empty entity and returns it as an Entity wrapper.
        /// </summary>
        /// <param name="context">The context to spawn the entity in.</param>
        /// <returns>The spawned entity.</returns>
        /// <exception cref="GoudEngineException">Thrown if entity spawning fails.</exception>
        public static Entity Spawn(this GoudContext context)
        {
            var entityId = context.SpawnEntity();
            return new Entity(context, entityId);
        }

        /// <summary>
        /// Spawns multiple empty entities in a batch and returns them as Entity wrappers.
        /// </summary>
        /// <param name="context">The context to spawn the entities in.</param>
        /// <param name="count">The number of entities to spawn.</param>
        /// <returns>An array of spawned entities.</returns>
        /// <exception cref="ArgumentOutOfRangeException">Thrown if count is negative or zero.</exception>
        /// <exception cref="GoudEngineException">Thrown if batch spawning fails.</exception>
        public static Entity[] SpawnBatch(this GoudContext context, int count)
        {
            var entityIds = context.SpawnEntitiesBatch(count);
            var entities = new Entity[entityIds.Length];

            for (int i = 0; i < entityIds.Length; i++)
            {
                entities[i] = new Entity(context, entityIds[i]);
            }

            return entities;
        }

        /// <summary>
        /// Wraps a raw entity ID in an Entity object for easier use.
        /// </summary>
        /// <param name="context">The context the entity belongs to.</param>
        /// <param name="entityId">The raw entity ID.</param>
        /// <returns>The wrapped entity.</returns>
        /// <exception cref="ArgumentException">Thrown if entity ID is invalid.</exception>
        public static Entity WrapEntity(this GoudContext context, GoudEntityId entityId)
        {
            return new Entity(context, entityId);
        }

        /// <summary>
        /// Despawns multiple entities in a batch.
        /// </summary>
        /// <param name="context">The context to despawn the entities from.</param>
        /// <param name="entities">The entities to despawn.</param>
        /// <returns>The number of entities successfully despawned.</returns>
        public static uint DespawnBatch(this GoudContext context, params Entity[] entities)
        {
            if (entities == null || entities.Length == 0)
            {
                return 0;
            }

            var entityIds = new GoudEntityId[entities.Length];
            for (int i = 0; i < entities.Length; i++)
            {
                entityIds[i] = entities[i].Id;
            }

            return context.DespawnEntitiesBatch(entityIds);
        }

        /// <summary>
        /// Checks if multiple entities are alive in a batch.
        /// </summary>
        /// <param name="context">The context to check entities in.</param>
        /// <param name="entities">The entities to check.</param>
        /// <returns>An array of booleans indicating which entities are alive.</returns>
        public static bool[] AreEntitiesAlive(this GoudContext context, params Entity[] entities)
        {
            if (entities == null || entities.Length == 0)
            {
                return Array.Empty<bool>();
            }

            var entityIds = new GoudEntityId[entities.Length];
            for (int i = 0; i < entities.Length; i++)
            {
                entityIds[i] = entities[i].Id;
            }

            return context.IsEntitiesAliveBatch(entityIds);
        }
    }
}

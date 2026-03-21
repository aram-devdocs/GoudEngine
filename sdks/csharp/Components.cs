using System;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace GoudEngine
{
    /// <summary>
    /// Provides static helpers for generic component operations, including
    /// FNV-1a type hashing that matches the engine's Rust implementation.
    /// </summary>
    public static class ComponentStore<T> where T : unmanaged
    {
        /// <summary>Cached FNV-1a hash of typeof(T).Name.</summary>
        public static readonly ulong Hash = ComponentStore.TypeHash(typeof(T).Name);
    }

    /// <summary>
    /// Non-generic companion with the raw FNV-1a hash function.
    /// </summary>
    public static class ComponentStore
    {
        /// <summary>
        /// Computes a 64-bit FNV-1a hash of the given type name.
        /// The result matches the Rust and Python codegen implementations.
        /// </summary>
        public static ulong TypeHash(string typeName)
        {
            ulong h = 0xcbf29ce484222325;
            foreach (byte b in System.Text.Encoding.UTF8.GetBytes(typeName))
            {
                h ^= b;
                h *= 0x100000001b3;
            }
            return h;
        }
    }

    /// <summary>
    /// Extension methods that add strongly-typed generic component operations
    /// to <see cref="GoudGame"/>.
    /// </summary>
    public static class ComponentExtensions
    {
        /// <summary>
        /// Registers component type <typeparamref name="T"/> with the engine.
        /// Must be called once per type before any other component operation.
        /// </summary>
        public static bool RegisterComponent<T>(this GoudGame game) where T : unmanaged
        {
            ulong hash = ComponentStore<T>.Hash;
            string name = typeof(T).Name;
            unsafe
            {
                byte[] nameBytes = System.Text.Encoding.UTF8.GetBytes(name);
                fixed (byte* np = nameBytes)
                {
                    return NativeMethods.goud_component_register_type(
                        hash, (IntPtr)np, (nuint)nameBytes.Length,
                        (nuint)sizeof(T), (nuint)(uint)AlignOf<T>());
                }
            }
        }

        /// <summary>
        /// Adds or overwrites component <typeparamref name="T"/> on the given entity.
        /// </summary>
        public static bool SetComponent<T>(this GoudGame game, Entity entity, T value) where T : unmanaged
        {
            ulong hash = ComponentStore<T>.Hash;
            unsafe
            {
                return game.ComponentAdd(entity, hash, (IntPtr)(&value), (nuint)sizeof(T));
            }
        }

        /// <summary>
        /// Returns a read-only reference to the component on the given entity,
        /// or <c>null</c> if the entity does not have this component type.
        /// The returned pointer is valid only until the next structural mutation.
        /// </summary>
        public static unsafe ref readonly T GetComponent<T>(this GoudGame game, Entity entity) where T : unmanaged
        {
            ulong hash = ComponentStore<T>.Hash;
            IntPtr ptr = game.ComponentGet(entity, hash);
            if (ptr == IntPtr.Zero)
                throw new InvalidOperationException(
                    $"Entity {entity} does not have component {typeof(T).Name}");
            return ref Unsafe.AsRef<T>((void*)ptr);
        }

        /// <summary>
        /// Returns a mutable reference to the component on the given entity.
        /// The returned pointer is valid only until the next structural mutation.
        /// </summary>
        public static unsafe ref T GetComponentMut<T>(this GoudGame game, Entity entity) where T : unmanaged
        {
            ulong hash = ComponentStore<T>.Hash;
            IntPtr ptr = game.ComponentGetMut(entity, hash);
            if (ptr == IntPtr.Zero)
                throw new InvalidOperationException(
                    $"Entity {entity} does not have component {typeof(T).Name}");
            return ref Unsafe.AsRef<T>((void*)ptr);
        }

        /// <summary>
        /// Checks whether the given entity has component <typeparamref name="T"/>.
        /// </summary>
        public static bool HasComponent<T>(this GoudGame game, Entity entity) where T : unmanaged
        {
            return game.ComponentHas(entity, ComponentStore<T>.Hash);
        }

        /// <summary>
        /// Removes component <typeparamref name="T"/> from the given entity.
        /// Returns <c>true</c> if the component was present and removed.
        /// </summary>
        public static bool RemoveComponent<T>(this GoudGame game, Entity entity) where T : unmanaged
        {
            return game.ComponentRemove(entity, ComponentStore<T>.Hash);
        }

        /// <summary>
        /// Returns a <see cref="ComponentQuery{T}"/> that can be iterated with
        /// <c>foreach</c> to visit every entity that has component
        /// <typeparamref name="T"/>. Zero heap allocations.
        /// </summary>
        public static ComponentQuery<T> Query<T>(this GoudGame game) where T : unmanaged
        {
            return new ComponentQuery<T>(game);
        }

        // Portable alignment helper. For unmanaged T the runtime lays out
        // the struct according to its natural alignment.
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        private static unsafe int AlignOf<T>() where T : unmanaged
        {
            // Use a helper struct to derive natural alignment.
            return sizeof(AlignHelper<T>) - sizeof(T);
        }

        [StructLayout(LayoutKind.Sequential)]
        private struct AlignHelper<T> where T : unmanaged
        {
            public byte Pad;
            public T Value;
        }
    }

    /// <summary>
    /// Stack-only query result that enumerates all entities with component
    /// <typeparamref name="T"/>. Obtain via
    /// <see cref="ComponentExtensions.Query{T}(GoudGame)"/>.
    /// <para>Data pointers are valid only until the next structural ECS mutation.</para>
    /// </summary>
    public ref struct ComponentQuery<T> where T : unmanaged
    {
        private readonly GoudGame _game;
        private readonly ulong _typeHash;

        internal ComponentQuery(GoudGame game)
        {
            _game = game;
            _typeHash = ComponentStore<T>.Hash;
        }

        /// <summary>Number of entities that match this query.</summary>
        public int Count => (int)_game.ComponentCount(_typeHash);

        /// <summary>Returns an enumerator for <c>foreach</c> usage.</summary>
        public Enumerator GetEnumerator() => new Enumerator(_game, _typeHash);

        /// <summary>
        /// Stack-only enumerator. Calls <c>ComponentGetAll</c> once during
        /// construction and then iterates the returned buffers.
        /// </summary>
        public ref struct Enumerator
        {
            private readonly int _count;
            private readonly IntPtr _entityBuf;
            private readonly IntPtr _dataPtrBuf;
            private int _index;

            internal unsafe Enumerator(GoudGame game, ulong typeHash)
            {
                uint count = game.ComponentCount(typeHash);
                _count = (int)count;
                _index = -1;

                if (count == 0)
                {
                    _entityBuf = IntPtr.Zero;
                    _dataPtrBuf = IntPtr.Zero;
                    return;
                }

                // Allocate native buffers for entity IDs and data pointers.
                _entityBuf = Marshal.AllocHGlobal((int)(count * sizeof(ulong)));
                _dataPtrBuf = Marshal.AllocHGlobal((int)(count * sizeof(IntPtr)));

                game.ComponentGetAll(typeHash, _entityBuf, _dataPtrBuf, count);
            }

            /// <summary>Advances to the next entity.</summary>
            public bool MoveNext()
            {
                _index++;
                return _index < _count;
            }

            /// <summary>The current entity/component pair.</summary>
            public unsafe ComponentRef<T> Current
            {
                get
                {
                    ulong bits = *((ulong*)_entityBuf + _index);
                    IntPtr dataPtr = *((IntPtr*)_dataPtrBuf + _index);
                    return new ComponentRef<T>(new Entity(bits), dataPtr);
                }
            }

            /// <summary>Frees native buffers allocated during enumeration.</summary>
            public void Dispose()
            {
                if (_entityBuf != IntPtr.Zero)
                    Marshal.FreeHGlobal(_entityBuf);
                if (_dataPtrBuf != IntPtr.Zero)
                    Marshal.FreeHGlobal(_dataPtrBuf);
            }
        }
    }

    /// <summary>
    /// A single entity/component pair returned during query iteration.
    /// Provides a <c>ref T</c> to the component data with zero copies.
    /// </summary>
    public readonly ref struct ComponentRef<T> where T : unmanaged
    {
        /// <summary>The entity that owns this component.</summary>
        public readonly Entity Entity;

        private readonly IntPtr _dataPtr;

        internal ComponentRef(Entity entity, IntPtr dataPtr)
        {
            Entity = entity;
            _dataPtr = dataPtr;
        }

        /// <summary>
        /// A managed reference to the component data. Valid only until the
        /// next structural ECS mutation.
        /// </summary>
        public unsafe ref T Value => ref Unsafe.AsRef<T>((void*)_dataPtr);
    }
}

using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
using CsBindgen;

namespace GoudEngine.Components
{
    /// <summary>
    /// Registry for component types in the GoudEngine ECS system.
    /// Handles registration of component types with the Rust FFI layer.
    /// </summary>
    public static class ComponentRegistry
    {
        private static readonly HashSet<ulong> _registeredTypes = new();
        private static readonly object _lock = new();

        /// <summary>
        /// Registers a component type with the engine if not already registered.
        /// </summary>
        /// <typeparam name="T">The component type to register.</typeparam>
        /// <exception cref="InvalidOperationException">Thrown if type lacks ComponentAttribute.</exception>
        public static void Register<T>() where T : struct, IComponent
        {
            var componentType = typeof(T);
            var attribute = GetComponentAttribute<T>();

            if (attribute == null)
            {
                throw new InvalidOperationException(
                    $"Component type {componentType.Name} must have [Component] attribute");
            }

            var typeId = attribute.TypeId;

            lock (_lock)
            {
                if (_registeredTypes.Contains(typeId))
                {
                    return; // Already registered
                }

                var name = componentType.Name;
                var size = attribute.SizeInBytes;
                var align = Marshal.SizeOf<T>(); // Use actual struct size for alignment

                unsafe
                {
                    var nameBytes = Encoding.UTF8.GetBytes(name);
                    fixed (byte* namePtr = nameBytes)
                    {
                        NativeMethods.goud_component_register_type(
                            typeId,
                            namePtr,
                            (nuint)nameBytes.Length,
                            (nuint)size,
                            (nuint)align
                        );
                    }
                }

                _registeredTypes.Add(typeId);
            }
        }

        /// <summary>
        /// Checks if a component type has been registered.
        /// </summary>
        /// <typeparam name="T">The component type to check.</typeparam>
        /// <returns>True if registered, false otherwise.</returns>
        public static bool IsRegistered<T>() where T : struct, IComponent
        {
            var attribute = GetComponentAttribute<T>();
            if (attribute == null)
            {
                return false;
            }

            lock (_lock)
            {
                return _registeredTypes.Contains(attribute.TypeId);
            }
        }

        /// <summary>
        /// Gets the ComponentAttribute for a type, or null if not present.
        /// </summary>
        internal static ComponentAttribute? GetComponentAttribute<T>() where T : struct, IComponent
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
        /// Clears the registration cache. Used for testing.
        /// </summary>
        internal static void ClearRegistrations()
        {
            lock (_lock)
            {
                _registeredTypes.Clear();
            }
        }
    }
}

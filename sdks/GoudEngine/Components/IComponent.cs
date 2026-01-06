using System;

namespace GoudEngine.Components
{
    /// <summary>
    /// Marker interface for all ECS components.
    /// Components are data containers that can be attached to entities.
    /// </summary>
    public interface IComponent
    {
        /// <summary>
        /// Gets the unique type ID hash for this component type.
        /// This is used by the FFI layer to identify component types.
        /// </summary>
        ulong TypeId { get; }

        /// <summary>
        /// Gets the size of this component in bytes.
        /// Must match the Rust struct size for FFI compatibility.
        /// </summary>
        int SizeInBytes { get; }
    }

    /// <summary>
    /// Attribute to mark component types and specify their type ID hash.
    /// The type ID should match the hash computed by the Rust side.
    /// </summary>
    [AttributeUsage(AttributeTargets.Struct, AllowMultiple = false, Inherited = false)]
    public sealed class ComponentAttribute : Attribute
    {
        /// <summary>
        /// Gets the type ID hash for this component.
        /// </summary>
        public ulong TypeId { get; }

        /// <summary>
        /// Gets the expected size in bytes for this component.
        /// Used for validation against the FFI layer.
        /// </summary>
        public int SizeInBytes { get; }

        /// <summary>
        /// Creates a new component attribute.
        /// </summary>
        /// <param name="typeId">The unique type ID hash.</param>
        /// <param name="sizeInBytes">The expected size in bytes.</param>
        public ComponentAttribute(ulong typeId, int sizeInBytes)
        {
            TypeId = typeId;
            SizeInBytes = sizeInBytes;
        }
    }
}

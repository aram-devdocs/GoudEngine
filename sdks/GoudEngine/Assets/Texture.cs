using GoudEngine.Core;

namespace GoudEngine.Assets;

/// <summary>
/// Represents a loaded texture asset.
/// </summary>
public class Texture
{
    /// <summary>
    /// The unique identifier for this texture.
    /// </summary>
    public TextureId Id { get; }

    /// <summary>
    /// The file path this texture was loaded from.
    /// </summary>
    public string Path { get; }

    internal Texture(TextureId id, string path)
    {
        Id = id;
        Path = path;
    }

    /// <summary>
    /// Implicit conversion to TextureId for convenience.
    /// </summary>
    public static implicit operator TextureId(Texture texture) => texture.Id;

    /// <summary>
    /// Implicit conversion to uint for backwards compatibility.
    /// </summary>
    public static implicit operator uint(Texture texture) => texture.Id.Value;

    public override string ToString() => $"Texture({Id}, \"{Path}\")";
}

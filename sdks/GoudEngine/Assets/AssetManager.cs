using System;
using System.Collections.Generic;
using GoudEngine.Core;

namespace GoudEngine.Assets;

/// <summary>
/// Manages game assets with caching support.
/// </summary>
public class AssetManager
{
    private readonly GoudGame _game;
    private readonly Dictionary<string, Texture> _textureCache = new();

    /// <summary>
    /// Creates a new asset manager for the specified game.
    /// </summary>
    public AssetManager(GoudGame game)
    {
        _game = game ?? throw new ArgumentNullException(nameof(game));
    }

    /// <summary>
    /// Loads a texture from the specified path.
    /// If the texture is already cached, returns the cached version.
    /// </summary>
    /// <param name="path">The path to the texture file.</param>
    /// <returns>The loaded texture.</returns>
    public Texture LoadTexture(string path)
    {
        if (string.IsNullOrEmpty(path))
            throw new ArgumentException("Path cannot be null or empty", nameof(path));

        // Check cache first
        if (_textureCache.TryGetValue(path, out var cached))
            return cached;

        // Load and cache
        var id = new TextureId(_game.CreateTexture(path));
        var texture = new Texture(id, path);
        _textureCache[path] = texture;
        return texture;
    }

    /// <summary>
    /// Gets a cached texture by path, or null if not loaded.
    /// </summary>
    public Texture? GetTexture(string path)
    {
        return _textureCache.TryGetValue(path, out var texture) ? texture : null;
    }

    /// <summary>
    /// Checks if a texture is already cached.
    /// </summary>
    public bool IsTextureCached(string path)
    {
        return _textureCache.ContainsKey(path);
    }

    /// <summary>
    /// Gets all cached textures.
    /// </summary>
    public IEnumerable<Texture> GetAllTextures()
    {
        return _textureCache.Values;
    }

    /// <summary>
    /// Clears the texture cache.
    /// Note: This does not unload textures from the engine.
    /// </summary>
    public void ClearCache()
    {
        _textureCache.Clear();
    }
}

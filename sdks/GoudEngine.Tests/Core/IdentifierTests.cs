using System;
using Xunit;
using GoudEngine.Core;

namespace GoudEngine.Tests.Core;

public class SpriteIdTests
{
    [Fact]
    public void Constructor_SetsValue()
    {
        var id = new SpriteId(42);
        Assert.Equal(42u, id.Value);
    }

    [Fact]
    public void Invalid_HasMaxValue()
    {
        var invalid = SpriteId.Invalid;
        Assert.Equal(uint.MaxValue, invalid.Value);
    }

    [Fact]
    public void IsValid_ReturnsTrue_ForNormalValues()
    {
        var id = new SpriteId(0);
        Assert.True(id.IsValid);

        var id2 = new SpriteId(100);
        Assert.True(id2.IsValid);
    }

    [Fact]
    public void IsValid_ReturnsFalse_ForInvalid()
    {
        Assert.False(SpriteId.Invalid.IsValid);
    }

    [Fact]
    public void ImplicitConversion_ToUint_Works()
    {
        var id = new SpriteId(42);
        uint value = id;
        Assert.Equal(42u, value);
    }

    [Fact]
    public void ExplicitConversion_FromUint_Works()
    {
        var id = (SpriteId)42u;
        Assert.Equal(42u, id.Value);
    }

    [Fact]
    public void Equality_WorksCorrectly()
    {
        var a = new SpriteId(10);
        var b = new SpriteId(10);
        var c = new SpriteId(20);

        Assert.True(a == b);
        Assert.False(a == c);
        Assert.True(a != c);
        Assert.True(a.Equals(b));
        Assert.False(a.Equals(c));
    }

    [Fact]
    public void Equals_Object_WorksCorrectly()
    {
        var a = new SpriteId(10);
        object b = new SpriteId(10);
        object c = new SpriteId(20);
        object notId = "not an id";

        Assert.True(a.Equals(b));
        Assert.False(a.Equals(c));
        Assert.False(a.Equals(notId));
        Assert.False(a.Equals(null));
    }

    [Fact]
    public void GetHashCode_IsSameForEqualIds()
    {
        var a = new SpriteId(10);
        var b = new SpriteId(10);
        Assert.Equal(a.GetHashCode(), b.GetHashCode());
    }

    [Fact]
    public void ToString_FormatsCorrectly()
    {
        var id = new SpriteId(42);
        Assert.Equal("Sprite(42)", id.ToString());
    }
}

public class TextureIdTests
{
    [Fact]
    public void Constructor_SetsValue()
    {
        var id = new TextureId(42);
        Assert.Equal(42u, id.Value);
    }

    [Fact]
    public void Invalid_HasMaxValue()
    {
        var invalid = TextureId.Invalid;
        Assert.Equal(uint.MaxValue, invalid.Value);
    }

    [Fact]
    public void IsValid_ReturnsTrue_ForNormalValues()
    {
        Assert.True(new TextureId(0).IsValid);
        Assert.True(new TextureId(100).IsValid);
    }

    [Fact]
    public void IsValid_ReturnsFalse_ForInvalid()
    {
        Assert.False(TextureId.Invalid.IsValid);
    }

    [Fact]
    public void ImplicitConversion_ToUint_Works()
    {
        var id = new TextureId(42);
        uint value = id;
        Assert.Equal(42u, value);
    }

    [Fact]
    public void ExplicitConversion_FromUint_Works()
    {
        var id = (TextureId)42u;
        Assert.Equal(42u, id.Value);
    }

    [Fact]
    public void Equality_WorksCorrectly()
    {
        var a = new TextureId(10);
        var b = new TextureId(10);
        var c = new TextureId(20);

        Assert.True(a == b);
        Assert.False(a == c);
        Assert.True(a.Equals(b));
    }

    [Fact]
    public void ToString_FormatsCorrectly()
    {
        var id = new TextureId(42);
        Assert.Equal("Texture(42)", id.ToString());
    }
}

public class ObjectIdTests
{
    [Fact]
    public void Constructor_SetsValue()
    {
        var id = new ObjectId(42);
        Assert.Equal(42u, id.Value);
    }

    [Fact]
    public void Invalid_HasMaxValue()
    {
        var invalid = ObjectId.Invalid;
        Assert.Equal(uint.MaxValue, invalid.Value);
    }

    [Fact]
    public void IsValid_ReturnsTrue_ForNormalValues()
    {
        Assert.True(new ObjectId(0).IsValid);
        Assert.True(new ObjectId(100).IsValid);
    }

    [Fact]
    public void IsValid_ReturnsFalse_ForInvalid()
    {
        Assert.False(ObjectId.Invalid.IsValid);
    }

    [Fact]
    public void ImplicitConversion_ToUint_Works()
    {
        var id = new ObjectId(42);
        uint value = id;
        Assert.Equal(42u, value);
    }

    [Fact]
    public void ExplicitConversion_FromUint_Works()
    {
        var id = (ObjectId)42u;
        Assert.Equal(42u, id.Value);
    }

    [Fact]
    public void Equality_WorksCorrectly()
    {
        var a = new ObjectId(10);
        var b = new ObjectId(10);
        var c = new ObjectId(20);

        Assert.True(a == b);
        Assert.False(a == c);
        Assert.True(a.Equals(b));
    }

    [Fact]
    public void ToString_FormatsCorrectly()
    {
        var id = new ObjectId(42);
        Assert.Equal("Object3D(42)", id.ToString());
    }
}

public class LightIdTests
{
    [Fact]
    public void Constructor_SetsValue()
    {
        var id = new LightId(42);
        Assert.Equal(42u, id.Value);
    }

    [Fact]
    public void Invalid_HasMaxValue()
    {
        var invalid = LightId.Invalid;
        Assert.Equal(uint.MaxValue, invalid.Value);
    }

    [Fact]
    public void IsValid_ReturnsTrue_ForNormalValues()
    {
        Assert.True(new LightId(0).IsValid);
        Assert.True(new LightId(100).IsValid);
    }

    [Fact]
    public void IsValid_ReturnsFalse_ForInvalid()
    {
        Assert.False(LightId.Invalid.IsValid);
    }

    [Fact]
    public void ImplicitConversion_ToUint_Works()
    {
        var id = new LightId(42);
        uint value = id;
        Assert.Equal(42u, value);
    }

    [Fact]
    public void ExplicitConversion_FromUint_Works()
    {
        var id = (LightId)42u;
        Assert.Equal(42u, id.Value);
    }

    [Fact]
    public void Equality_WorksCorrectly()
    {
        var a = new LightId(10);
        var b = new LightId(10);
        var c = new LightId(20);

        Assert.True(a == b);
        Assert.False(a == c);
        Assert.True(a.Equals(b));
    }

    [Fact]
    public void ToString_FormatsCorrectly()
    {
        var id = new LightId(42);
        Assert.Equal("Light(42)", id.ToString());
    }
}

public class TiledMapIdTests
{
    [Fact]
    public void Constructor_SetsValue()
    {
        var id = new TiledMapId(42);
        Assert.Equal(42u, id.Value);
    }

    [Fact]
    public void Invalid_HasMaxValue()
    {
        var invalid = TiledMapId.Invalid;
        Assert.Equal(uint.MaxValue, invalid.Value);
    }

    [Fact]
    public void IsValid_ReturnsTrue_ForNormalValues()
    {
        Assert.True(new TiledMapId(0).IsValid);
        Assert.True(new TiledMapId(100).IsValid);
    }

    [Fact]
    public void IsValid_ReturnsFalse_ForInvalid()
    {
        Assert.False(TiledMapId.Invalid.IsValid);
    }

    [Fact]
    public void ImplicitConversion_ToUint_Works()
    {
        var id = new TiledMapId(42);
        uint value = id;
        Assert.Equal(42u, value);
    }

    [Fact]
    public void ExplicitConversion_FromUint_Works()
    {
        var id = (TiledMapId)42u;
        Assert.Equal(42u, id.Value);
    }

    [Fact]
    public void Equality_WorksCorrectly()
    {
        var a = new TiledMapId(10);
        var b = new TiledMapId(10);
        var c = new TiledMapId(20);

        Assert.True(a == b);
        Assert.False(a == c);
        Assert.True(a.Equals(b));
    }

    [Fact]
    public void ToString_FormatsCorrectly()
    {
        var id = new TiledMapId(42);
        Assert.Equal("TiledMap(42)", id.ToString());
    }
}

// Tests to verify type safety - different ID types should not be interchangeable at compile time
public class IdTypeSafetyTests
{
    [Fact]
    public void DifferentIdTypes_HaveDifferentTypes()
    {
        // Verify they are distinct types
        Assert.NotEqual(typeof(SpriteId), typeof(TextureId));
        Assert.NotEqual(typeof(SpriteId), typeof(ObjectId));
        Assert.NotEqual(typeof(SpriteId), typeof(LightId));
        Assert.NotEqual(typeof(TextureId), typeof(ObjectId));
    }

    [Fact]
    public void SameValue_DifferentTypes_NotEqual()
    {
        var spriteId = new SpriteId(42);
        var textureId = new TextureId(42);

        // They have the same underlying value
        Assert.Equal(spriteId.Value, textureId.Value);

        // But are not equal as objects (different types)
        Assert.False(spriteId.Equals((object)textureId));
    }
}

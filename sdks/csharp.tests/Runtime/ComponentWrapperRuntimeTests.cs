using System;
using System.Linq;
using System.Reflection;
using GoudEngine;
using Xunit;

namespace GoudEngine.Tests.Runtime;

public class ComponentWrapperRuntimeTests
{
    [Fact]
    public void Component_Property_Accessors_RoundTrip()
    {
        AssertPropertyRoundTrip<Transform2D>();
        AssertPropertyRoundTrip<Sprite>();
        AssertPropertyRoundTrip<SpriteAnimator>();
        AssertPropertyRoundTrip<Text>();
    }

    [Fact]
    public void Transform2D_Statics_Methods_And_Builder_Execute()
    {
        var transform = Transform2D.Default();
        Assert.Equal(new Vec2(0f, 0f).ToString(), transform.GetPosition().ToString());

        transform = Transform2D.FromPosition(2f, 3f);
        Assert.Equal(new Vec2(2f, 3f).ToString(), transform.GetPosition().ToString());

        transform = Transform2D.FromRotationDegrees(90f);
        Assert.Equal(90f, transform.GetRotationDegrees(), 2);

        transform = Transform2D.FromScale(2f, 4f);
        Assert.Equal(new Vec2(2f, 4f).ToString(), transform.GetScale().ToString());

        transform = Transform2D.New(1f, 2f, 0f, 3f, 4f);
        transform.Translate(2f, -1f);
        transform.TranslateLocal(1f, 0f);
        transform.SetPosition(5f, 6f);
        transform.Rotate(MathF.PI / 4f);
        transform.RotateDegrees(45f);
        transform.SetRotation(MathF.PI / 2f);
        transform.SetRotationDegrees(180f);
        transform.LookAtTarget(10f, 6f);
        transform.SetScale(7f, 8f);
        transform.SetScaleUniform(2f);
        transform.ScaleBy(1.5f, 2f);

        Assert.NotEqual(Vec2.Zero().ToString(), transform.Forward().ToString());
        Assert.NotEqual(Vec2.Zero().ToString(), transform.Right().ToString());
        Assert.NotEqual(Vec2.Zero().ToString(), transform.Backward().ToString());
        Assert.NotEqual(Vec2.Zero().ToString(), transform.Left().ToString());
        Assert.NotNull(transform.Matrix().ToString());
        Assert.NotNull(transform.MatrixInverse().ToString());
        Assert.NotNull(transform.TransformPoint(1f, 1f).ToString());
        Assert.NotNull(transform.TransformDirection(1f, 0f).ToString());
        Assert.NotNull(transform.InverseTransformPoint(1f, 1f).ToString());
        Assert.NotNull(transform.InverseTransformDirection(1f, 0f).ToString());
        Assert.Equal(
            Transform2D.FromPositionRotation(0f, 0f, 0f).Lerp(Transform2D.FromPositionRotation(10f, 0f, MathF.PI), 0.5f).ToString(),
            Transform2D.FromPositionRotation(0f, 0f, 0f).Lerp(Transform2D.FromPositionRotation(10f, 0f, MathF.PI), 0.5f).ToString()
        );
        Assert.InRange(Transform2D.NormalizeAngle(10f), -MathF.PI, MathF.PI);
        Assert.Contains("Transform2D(", transform.ToString(), StringComparison.Ordinal);

        using var builder = Transform2DBuilder.New()
            .WithPosition(3f, 4f)
            .WithRotationDegrees(90f)
            .WithScale(2f, 3f)
            .WithScaleUniform(2f)
            .LookingAt(10f, 10f)
            .Translate(1f, 2f)
            .Rotate(0.5f)
            .ScaleBy(0.5f, 0.5f);
        var built = builder.Build();
        Assert.NotEqual(Vec2.Zero().ToString(), built.GetScale().ToString());

        using var positionedBuilder = Transform2DBuilder.AtPosition(9f, 11f);
        Assert.Equal(new Vec2(9f, 11f).ToString(), positionedBuilder.Build().GetPosition().ToString());

        var lookAt = Transform2D.LookAt(0f, 0f, 0f, 10f);
        Assert.InRange(lookAt.GetRotationDegrees(), 80f, 100f);
    }

    [Fact]
    public void Sprite_Methods_And_Builder_Execute()
    {
        var sprite = Sprite.Default();
        sprite.SetColor(0.1f, 0.2f, 0.3f, 0.4f);
        Assert.Equal("Color(0.1, 0.2, 0.3, 0.4)", sprite.GetColor().ToString());

        sprite = sprite.WithColor(0.6f, 0.7f, 0.8f, 0.9f);
        Assert.Equal(0.9f, sprite.GetAlpha(), 3);

        sprite.SetAlpha(0.5f);
        Assert.Equal(0.5f, sprite.GetAlpha(), 3);

        sprite.SetSourceRect(1f, 2f, 3f, 4f);
        Assert.True(sprite.HasSourceRect());
        Assert.Equal("Rect(1, 2, 3, 4)", sprite.GetSourceRect().ToString());
        sprite = sprite.WithSourceRect(5f, 6f, 7f, 8f);
        Assert.Equal("Rect(5, 6, 7, 8)", sprite.GetSourceRect().ToString());
        sprite.ClearSourceRect();
        Assert.False(sprite.HasSourceRect());

        sprite.SetFlipX(true);
        sprite.SetFlipY(true);
        Assert.True(sprite.GetFlipX());
        Assert.True(sprite.GetFlipY());
        Assert.True(sprite.IsFlipped());
        sprite.SetFlip(false, false);
        Assert.False(sprite.IsFlipped());
        Assert.True(sprite.WithFlipX(true).GetFlipX());
        Assert.True(sprite.WithFlipY(true).GetFlipY());
        Assert.True(sprite.WithFlip(true, true).IsFlipped());

        sprite.SetAnchor(0.25f, 0.75f);
        Assert.Equal(new Vec2(0.25f, 0.75f).ToString(), sprite.GetAnchor().ToString());
        Assert.Equal(new Vec2(0.5f, 0.5f).ToString(), sprite.WithAnchor(0.5f, 0.5f).GetAnchor().ToString());

        sprite.SetCustomSize(64f, 32f);
        Assert.True(sprite.HasCustomSize());
        Assert.Equal(new Vec2(64f, 32f).ToString(), sprite.GetCustomSize().ToString());
        Assert.Equal(new Vec2(10f, 20f).ToString(), sprite.WithCustomSize(10f, 20f).GetCustomSize().ToString());
        Assert.Equal(new Vec2(10f, 20f).ToString(), sprite.WithCustomSize(10f, 20f).SizeOrRect().ToString());
        sprite.ClearCustomSize();
        Assert.False(sprite.HasCustomSize());

        sprite.SetTexture(77UL);
        Assert.Equal(77UL, sprite.GetTexture());
        Assert.Equal(99UL, Sprite.New(99UL).GetTexture());
        Assert.Contains("Sprite(", sprite.ToString(), StringComparison.Ordinal);

        using var defaultBuilder = SpriteBuilder.Default()
            .WithTexture(44UL)
            .WithColor(0.3f, 0.4f, 0.5f, 1f)
            .WithAlpha(0.8f)
            .WithSourceRect(1f, 2f, 3f, 4f)
            .WithFlip(true, false)
            .WithFlipX(true)
            .WithFlipY(true)
            .WithAnchor(0.2f, 0.8f)
            .WithCustomSize(16f, 24f);
        Assert.True(defaultBuilder.Build().HasCustomSize());

        using var builder = SpriteBuilder.New(12UL);
        var built = builder.WithTexture(13UL).Build();
        Assert.Equal(13UL, built.GetTexture());
    }

    [Fact]
    public void Text_And_SpriteAnimator_Methods_Execute()
    {
        var text = Text.Default();
        text.SetFontSize(32f);
        Assert.Equal(32f, text.GetFontSize(), 3);
        text.SetColor(0.1f, 0.2f, 0.3f, 0.4f);
        Assert.Equal(0.1f, text.GetColorR(), 3);
        Assert.Equal(0.2f, text.GetColorG(), 3);
        Assert.Equal(0.3f, text.GetColorB(), 3);
        Assert.Equal(0.4f, text.GetColorA(), 3);
        text.SetAlignment((byte)TextAlignment.Center);
        Assert.Equal((byte)TextAlignment.Center, text.GetAlignment());
        text.SetMaxWidth(200f);
        Assert.True(text.HasMaxWidth());
        Assert.Equal(200f, text.GetMaxWidth(), 3);
        text.ClearMaxWidth();
        Assert.False(text.HasMaxWidth());
        text.SetLineSpacing(1.75f);
        Assert.Equal(1.75f, text.GetLineSpacing(), 3);
        Assert.Equal(55UL, Text.New(55UL).FontHandle);
        Assert.Contains("Text(", text.ToString(), StringComparison.Ordinal);

        using var clipBuilder = SpriteAnimatorBuilder.New(0.1f, PlaybackMode.Loop)
            .AddFrame(0f, 0f, 16f, 16f)
            .AddFrame(16f, 0f, 16f, 16f);
        var animator = clipBuilder.Build();
        Assert.True(animator.IsPlaying());
        Assert.False(animator.IsFinished());
        Assert.Equal(animator.CurrentFrame, animator.GetCurrentFrame());
        Assert.Contains("SpriteAnimator(", animator.ToString(), StringComparison.Ordinal);
    }

    private static void AssertPropertyRoundTrip<T>() where T : struct
    {
        object boxed = new T();
        foreach (var property in typeof(T).GetProperties(BindingFlags.Instance | BindingFlags.Public)
                     .Where(candidate => candidate.CanRead && candidate.CanWrite && candidate.GetIndexParameters().Length == 0))
        {
            var value = CreateSample(property.PropertyType);
            property.SetValue(boxed, value);
            Assert.Equal(value, property.GetValue(boxed));
        }

        Assert.Contains(typeof(T).Name, boxed.ToString(), StringComparison.Ordinal);
    }

    private static object CreateSample(Type type)
    {
        if (type == typeof(float))
        {
            return 1.25f;
        }

        if (type == typeof(bool))
        {
            return true;
        }

        if (type == typeof(byte))
        {
            return (byte)2;
        }

        if (type == typeof(uint))
        {
            return 3U;
        }

        if (type == typeof(int))
        {
            return 5;
        }

        if (type == typeof(ulong))
        {
            return 4UL;
        }

        if (type.IsEnum)
        {
            return Enum.GetValues(type).GetValue(0)!;
        }

        throw new InvalidOperationException($"Unsupported property type: {type.FullName}");
    }
}

using System;
using System.IO;
using System.Reflection;
using GoudEngine;
using Xunit;

namespace GoudEngine.Tests.Ui;

public class UiManagerApiTests
{
    [Fact]
    public void UiManager_Exposes_LowLevel_Methods()
    {
        var t = typeof(UiManager);

        AssertMethod(t, "Update", typeof(void));
        AssertMethod(t, "Render", typeof(void));
        AssertMethod(t, "NodeCount", typeof(uint));
        AssertMethod(t, "CreateNode", typeof(ulong), typeof(int));
        AssertMethod(t, "RemoveNode", typeof(int), typeof(ulong));
        AssertMethod(t, "SetParent", typeof(int), typeof(ulong), typeof(ulong));
        AssertMethod(t, "GetParent", typeof(ulong), typeof(ulong));
        AssertMethod(t, "GetChildCount", typeof(uint), typeof(ulong));
        AssertMethod(t, "GetChildAt", typeof(ulong), typeof(ulong), typeof(uint));
        AssertMethod(t, "SetWidget", typeof(int), typeof(ulong), typeof(int));
        AssertMethod(t, "SetStyle", typeof(int), typeof(ulong), typeof(UiStyle));
        AssertMethod(t, "SetLabelText", typeof(int), typeof(ulong), typeof(string));
        AssertMethod(t, "SetButtonEnabled", typeof(int), typeof(ulong), typeof(bool));
        AssertMethod(t, "SetImageTexturePath", typeof(int), typeof(ulong), typeof(string));
        AssertMethod(t, "SetSlider", typeof(int), typeof(ulong), typeof(float), typeof(float), typeof(float), typeof(bool));
        AssertMethod(t, "EventCount", typeof(uint));
        AssertMethod(t, "EventRead", typeof(UiEvent?), typeof(uint));
        Assert.Null(t.GetMethod("SetEventCallback"));
    }

    [Fact]
    public void UiManager_Exposes_Convenience_Methods()
    {
        var t = typeof(UiManager);

        AssertMethod(t, "CreatePanel", typeof(ulong));
        AssertMethod(t, "CreateLabel", typeof(ulong), typeof(string));
        AssertMethod(t, "CreateButton", typeof(ulong), typeof(bool));
        AssertMethod(t, "CreateImage", typeof(ulong), typeof(string));
        AssertMethod(t, "CreateSlider", typeof(ulong), typeof(float), typeof(float), typeof(float), typeof(bool));
    }

    [Fact]
    public void UiValueTypes_Are_Present_For_UiManager_API()
    {
        Assert.Equal(typeof(UiStyle), typeof(UiManager).GetMethod("SetStyle")!.GetParameters()[1].ParameterType);
        Assert.Equal(typeof(UiEvent?), typeof(UiManager).GetMethod("EventRead")!.ReturnType);
        Assert.Equal(typeof(string), typeof(UiStyle).GetField("FontFamily")!.FieldType);
        Assert.Equal(typeof(string), typeof(UiStyle).GetField("TexturePath")!.FieldType);
        Assert.Null(typeof(UiStyle).GetField("FontFamilyPtr"));
        Assert.Null(typeof(UiStyle).GetField("TexturePathPtr"));
        Assert.Null(typeof(UiStyle).GetField("FontFamilyLen"));
        Assert.Null(typeof(UiStyle).GetField("TexturePathLen"));
    }

    [Fact]
    public void Generated_UiManager_Source_Uses_Managed_String_Marshalling()
    {
        string root = Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "../../../../"));
        string uiManagerSrc = File.ReadAllText(Path.Combine(root, "sdks/csharp/generated/Core/UiManager.g.cs"));
        string uiStyleSrc = File.ReadAllText(Path.Combine(root, "sdks/csharp/generated/Math/UiStyle.g.cs"));

        Assert.Contains("string? FontFamily;", uiStyleSrc);
        Assert.Contains("string? TexturePath;", uiStyleSrc);
        Assert.DoesNotContain("IntPtr FontFamilyPtr;", uiStyleSrc);
        Assert.DoesNotContain("IntPtr TexturePathPtr;", uiStyleSrc);
        Assert.DoesNotContain("nuint FontFamilyLen;", uiStyleSrc);
        Assert.DoesNotContain("nuint TexturePathLen;", uiStyleSrc);
        Assert.DoesNotContain("SetEventCallback", uiManagerSrc);
        Assert.Contains("Encoding.UTF8.GetBytes(style.FontFamily ?? string.Empty)", uiManagerSrc);
        Assert.Contains("Encoding.UTF8.GetBytes(style.TexturePath ?? string.Empty)", uiManagerSrc);
        Assert.Contains("fixed (byte* fontFamilyPtr = fontFamilyBytes)", uiManagerSrc);
        Assert.Contains("fixed (byte* texturePathPtr = texturePathBytes)", uiManagerSrc);
    }

    private static void AssertMethod(Type type, string name, Type returnType, params Type[] parameterTypes)
    {
        MethodInfo? method = type.GetMethod(name, parameterTypes);
        Assert.NotNull(method);
        Assert.Equal(returnType, method!.ReturnType);
    }
}

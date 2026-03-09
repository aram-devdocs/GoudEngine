using System.Reflection;
using Xunit;
using GoudEngine;

namespace GoudEngine.Tests.Scene
{
    /// <summary>
    /// Verifies generated scene wrapper API names for SDK ergonomics.
    /// These tests are reflection-only and do not require the native library.
    /// </summary>
    public class SceneWrapperApiTests
    {
        [Fact]
        public void GoudContext_Exposes_Idiomatic_Scene_Wrappers()
        {
            var t = typeof(GoudContext);

            Assert.NotNull(t.GetMethod("LoadScene", [typeof(string), typeof(string)]));
            Assert.NotNull(t.GetMethod("UnloadScene", [typeof(string)]));
            Assert.NotNull(t.GetMethod("SetActiveScene", [typeof(uint), typeof(bool)]));
        }

        [Fact]
        public void GoudContext_Keeps_Legacy_Scene_APIs()
        {
            var t = typeof(GoudContext);

            Assert.NotNull(t.GetMethod("SceneCreate", [typeof(string)]));
            Assert.NotNull(t.GetMethod("SceneDestroy", [typeof(uint)]));
            Assert.NotNull(t.GetMethod("SceneSetCurrent", [typeof(uint)]));
        }
    }
}

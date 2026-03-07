using System;
using Xunit;
using GoudEngine;

namespace GoudEngine.Tests.Errors
{
    /// <summary>
    /// Tests for GoudEngine C# exception hierarchy.
    /// Verifies that exception classes have correct properties and
    /// that subclass dispatch produces the expected types.
    /// These tests do not require the native library.
    /// </summary>
    public class ErrorTypesTests
    {
        [Fact]
        public void GoudException_Constructs_WithExpectedProperties()
        {
            var ex = new GoudException(
                errorCode: 1,
                message: "context not initialised",
                category: "Context",
                subsystem: "engine",
                operation: "init",
                recovery: RecoveryClass.Fatal,
                recoveryHint: "Call the initialization function first"
            );

            Assert.Equal(1, ex.ErrorCode);
            Assert.Equal("context not initialised", ex.Message);
            Assert.Equal("Context", ex.Category);
            Assert.Equal("engine", ex.Subsystem);
            Assert.Equal("init", ex.Operation);
            Assert.Equal(RecoveryClass.Fatal, ex.Recovery);
            Assert.Equal("Call the initialization function first", ex.RecoveryHint);
        }

        [Fact]
        public void GoudException_IsException()
        {
            var ex = new GoudException(1, "msg", "Context", "", "", RecoveryClass.Recoverable, "");
            Assert.IsAssignableFrom<Exception>(ex);
        }

        [Fact]
        public void RecoveryClass_HasExpectedValues()
        {
            Assert.Equal(0, (int)RecoveryClass.Recoverable);
            Assert.Equal(1, (int)RecoveryClass.Fatal);
            Assert.Equal(2, (int)RecoveryClass.Degraded);
        }

        [Fact]
        public void GoudContextException_IsSubclassOfGoudException()
        {
            var ex = new GoudContextException(1, "ctx", "engine", "init", RecoveryClass.Fatal, "");
            Assert.IsAssignableFrom<GoudException>(ex);
            Assert.Equal("Context", ex.Category);
        }

        [Fact]
        public void GoudResourceException_IsSubclassOfGoudException()
        {
            var ex = new GoudResourceException(100, "file not found", "assets", "load", RecoveryClass.Recoverable, "");
            Assert.IsAssignableFrom<GoudException>(ex);
            Assert.Equal("Resource", ex.Category);
        }

        [Fact]
        public void GoudGraphicsException_IsSubclassOfGoudException()
        {
            var ex = new GoudGraphicsException(200, "shader failed", "renderer", "compile", RecoveryClass.Fatal, "");
            Assert.IsAssignableFrom<GoudException>(ex);
            Assert.Equal("Graphics", ex.Category);
        }

        [Fact]
        public void GoudEntityException_IsSubclassOfGoudException()
        {
            var ex = new GoudEntityException(300, "entity missing", "ecs", "query", RecoveryClass.Recoverable, "");
            Assert.IsAssignableFrom<GoudException>(ex);
            Assert.Equal("Entity", ex.Category);
        }

        [Fact]
        public void GoudInputException_IsSubclassOfGoudException()
        {
            var ex = new GoudInputException(400, "device missing", "input", "poll", RecoveryClass.Recoverable, "");
            Assert.IsAssignableFrom<GoudException>(ex);
            Assert.Equal("Input", ex.Category);
        }

        [Fact]
        public void GoudSystemException_IsSubclassOfGoudException()
        {
            var ex = new GoudSystemException(500, "window failed", "platform", "create", RecoveryClass.Fatal, "");
            Assert.IsAssignableFrom<GoudException>(ex);
            Assert.Equal("System", ex.Category);
        }

        [Fact]
        public void GoudProviderException_IsSubclassOfGoudException()
        {
            var ex = new GoudProviderException(600, "provider down", "provider", "connect", RecoveryClass.Recoverable, "");
            Assert.IsAssignableFrom<GoudException>(ex);
            Assert.Equal("Provider", ex.Category);
        }

        [Fact]
        public void GoudInternalException_IsSubclassOfGoudException()
        {
            var ex = new GoudInternalException(900, "engine bug", "core", "internal", RecoveryClass.Fatal, "");
            Assert.IsAssignableFrom<GoudException>(ex);
            Assert.Equal("Internal", ex.Category);
        }

        [Fact]
        public void GoudContextException_CategoryIsContext()
        {
            var ex = new GoudContextException(1, "msg", "", "", RecoveryClass.Recoverable, "");
            Assert.Equal("Context", ex.Category);
        }

        [Fact]
        public void GoudResourceException_CategoryIsResource()
        {
            var ex = new GoudResourceException(100, "msg", "", "", RecoveryClass.Recoverable, "");
            Assert.Equal("Resource", ex.Category);
        }

        [Fact]
        public void GoudGraphicsException_CategoryIsGraphics()
        {
            var ex = new GoudGraphicsException(200, "msg", "", "", RecoveryClass.Recoverable, "");
            Assert.Equal("Graphics", ex.Category);
        }

        [Fact]
        public void GoudEntityException_CategoryIsEntity()
        {
            var ex = new GoudEntityException(300, "msg", "", "", RecoveryClass.Recoverable, "");
            Assert.Equal("Entity", ex.Category);
        }

        [Fact]
        public void GoudInputException_CategoryIsInput()
        {
            var ex = new GoudInputException(400, "msg", "", "", RecoveryClass.Recoverable, "");
            Assert.Equal("Input", ex.Category);
        }

        [Fact]
        public void GoudSystemException_CategoryIsSystem()
        {
            var ex = new GoudSystemException(500, "msg", "", "", RecoveryClass.Recoverable, "");
            Assert.Equal("System", ex.Category);
        }

        [Fact]
        public void GoudProviderException_CategoryIsProvider()
        {
            var ex = new GoudProviderException(600, "msg", "", "", RecoveryClass.Recoverable, "");
            Assert.Equal("Provider", ex.Category);
        }

        [Fact]
        public void GoudInternalException_CategoryIsInternal()
        {
            var ex = new GoudInternalException(900, "msg", "", "", RecoveryClass.Recoverable, "");
            Assert.Equal("Internal", ex.Category);
        }
    }
}

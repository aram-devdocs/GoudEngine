using System;
using Xunit;
using GoudEngine.Core;

namespace GoudEngine.Tests.Core
{
    /// <summary>
    /// Tests for GoudEngine exception types and error handling.
    /// </summary>
    public class ExceptionTests
    {
        // ====================================================================
        // GoudEngineException Tests
        // ====================================================================

        [Fact]
        public void GoudEngineException_SetsErrorCode()
        {
            var ex = new GoudEngineException(42, "Test error");
            Assert.Equal(42, ex.ErrorCode);
            Assert.Equal("Test error", ex.Message);
        }

        [Fact]
        public void GoudEngineException_WithInnerException()
        {
            var inner = new InvalidOperationException("Inner");
            var ex = new GoudEngineException(100, "Outer", inner);

            Assert.Equal(100, ex.ErrorCode);
            Assert.Equal("Outer", ex.Message);
            Assert.Same(inner, ex.InnerException);
        }

        [Fact]
        public void GoudEngineException_CategoryFromErrorCode()
        {
            Assert.Equal(ErrorCategory.Success, new GoudEngineException(0, "").Category);
            Assert.Equal(ErrorCategory.Context, new GoudEngineException(1, "").Category);
            Assert.Equal(ErrorCategory.Context, new GoudEngineException(99, "").Category);
            Assert.Equal(ErrorCategory.Resource, new GoudEngineException(100, "").Category);
            Assert.Equal(ErrorCategory.Resource, new GoudEngineException(199, "").Category);
            Assert.Equal(ErrorCategory.Graphics, new GoudEngineException(200, "").Category);
            Assert.Equal(ErrorCategory.Graphics, new GoudEngineException(299, "").Category);
            Assert.Equal(ErrorCategory.Entity, new GoudEngineException(300, "").Category);
            Assert.Equal(ErrorCategory.Entity, new GoudEngineException(399, "").Category);
            Assert.Equal(ErrorCategory.Input, new GoudEngineException(400, "").Category);
            Assert.Equal(ErrorCategory.Input, new GoudEngineException(499, "").Category);
            Assert.Equal(ErrorCategory.System, new GoudEngineException(500, "").Category);
            Assert.Equal(ErrorCategory.System, new GoudEngineException(599, "").Category);
            Assert.Equal(ErrorCategory.Internal, new GoudEngineException(900, "").Category);
            Assert.Equal(ErrorCategory.Internal, new GoudEngineException(999, "").Category);
            Assert.Equal(ErrorCategory.Unknown, new GoudEngineException(600, "").Category);
        }

        [Fact]
        public void GoudEngineException_ToStringIncludesErrorCode()
        {
            var ex = new GoudEngineException(42, "Test error");
            var str = ex.ToString();

            Assert.Contains("[GOUD-42]", str);
            Assert.Contains("Test error", str);
            Assert.Contains("Unknown", str); // Category
        }

        // ====================================================================
        // ContextException Tests
        // ====================================================================

        [Fact]
        public void ContextException_InheritsFromGoudEngineException()
        {
            var ex = new ContextException(1, "Context error");
            Assert.IsAssignableFrom<GoudEngineException>(ex);
            Assert.Equal(1, ex.ErrorCode);
            Assert.Equal(ErrorCategory.Context, ex.Category);
        }

        [Fact]
        public void ContextException_WithInnerException()
        {
            var inner = new Exception("Inner");
            var ex = new ContextException(2, "Outer", inner);

            Assert.Equal(2, ex.ErrorCode);
            Assert.Same(inner, ex.InnerException);
        }

        // ====================================================================
        // ResourceException Tests
        // ====================================================================

        [Fact]
        public void ResourceException_StoresResourcePath()
        {
            var ex = new ResourceException(100, "Load failed", "assets/texture.png");

            Assert.Equal(100, ex.ErrorCode);
            Assert.Equal("assets/texture.png", ex.ResourcePath);
            Assert.Equal(ErrorCategory.Resource, ex.Category);
        }

        [Fact]
        public void ResourceException_ToStringIncludesPath()
        {
            var ex = new ResourceException(101, "Not found", "assets/missing.png");
            var str = ex.ToString();

            Assert.Contains("[GOUD-101]", str);
            Assert.Contains("Not found", str);
            Assert.Contains("assets/missing.png", str);
        }

        [Fact]
        public void ResourceException_WithoutPath()
        {
            var ex = new ResourceException(102, "Generic error");
            Assert.Null(ex.ResourcePath);

            var str = ex.ToString();
            Assert.DoesNotContain("Resource:", str);
        }

        // ====================================================================
        // GraphicsException Tests
        // ====================================================================

        [Fact]
        public void GraphicsException_InheritsCorrectly()
        {
            var ex = new GraphicsException(200, "Shader compile failed");
            Assert.IsAssignableFrom<GoudEngineException>(ex);
            Assert.Equal(200, ex.ErrorCode);
            Assert.Equal(ErrorCategory.Graphics, ex.Category);
        }

        // ====================================================================
        // EntityException Tests
        // ====================================================================

        [Fact]
        public void EntityException_StoresEntityId()
        {
            var entityId = new GoudEntityId(12345);
            var ex = new EntityException(300, "Entity not found", entityId);

            Assert.Equal(300, ex.ErrorCode);
            Assert.Equal(entityId, ex.EntityId);
            Assert.Equal(ErrorCategory.Entity, ex.Category);
        }

        [Fact]
        public void EntityException_ToStringIncludesEntityId()
        {
            var entityId = new GoudEntityId(12345);
            var ex = new EntityException(301, "Entity despawned", entityId);
            var str = ex.ToString();

            Assert.Contains("[GOUD-301]", str);
            Assert.Contains("Entity despawned", str);
            Assert.Contains("12345", str); // Entity ID in string
        }

        [Fact]
        public void EntityException_WithoutEntityId()
        {
            var ex = new EntityException(302, "Generic entity error");
            Assert.Null(ex.EntityId);

            var str = ex.ToString();
            Assert.DoesNotContain("Entity:", str);
        }

        // ====================================================================
        // InputException Tests
        // ====================================================================

        [Fact]
        public void InputException_InheritsCorrectly()
        {
            var ex = new InputException(400, "Input device not found");
            Assert.IsAssignableFrom<GoudEngineException>(ex);
            Assert.Equal(400, ex.ErrorCode);
            Assert.Equal(ErrorCategory.Input, ex.Category);
        }

        // ====================================================================
        // SystemException Tests (GoudEngine.Core.SystemException)
        // ====================================================================

        [Fact]
        public void SystemException_InheritsCorrectly()
        {
            var ex = new Core.SystemException(500, "Window creation failed");
            Assert.IsAssignableFrom<GoudEngineException>(ex);
            Assert.Equal(500, ex.ErrorCode);
            Assert.Equal(ErrorCategory.System, ex.Category);
        }

        // ====================================================================
        // InternalException Tests
        // ====================================================================

        [Fact]
        public void InternalException_InheritsCorrectly()
        {
            var ex = new InternalException(900, "Internal error");
            Assert.IsAssignableFrom<GoudEngineException>(ex);
            Assert.Equal(900, ex.ErrorCode);
            Assert.Equal(ErrorCategory.Internal, ex.Category);
        }

        // ====================================================================
        // ErrorHelper Tests
        // ====================================================================

        [Fact]
        public void ErrorHelper_CreateException_Context()
        {
            var ex = ErrorHelper.CreateException(1, "Context error");
            Assert.IsType<ContextException>(ex);
            Assert.Equal(1, ex.ErrorCode);
        }

        [Fact]
        public void ErrorHelper_CreateException_Resource()
        {
            var ex = ErrorHelper.CreateException(100, "Resource error");
            Assert.IsType<ResourceException>(ex);
            Assert.Equal(100, ex.ErrorCode);
        }

        [Fact]
        public void ErrorHelper_CreateException_Graphics()
        {
            var ex = ErrorHelper.CreateException(200, "Graphics error");
            Assert.IsType<GraphicsException>(ex);
            Assert.Equal(200, ex.ErrorCode);
        }

        [Fact]
        public void ErrorHelper_CreateException_Entity()
        {
            var ex = ErrorHelper.CreateException(300, "Entity error");
            Assert.IsType<EntityException>(ex);
            Assert.Equal(300, ex.ErrorCode);
        }

        [Fact]
        public void ErrorHelper_CreateException_Input()
        {
            var ex = ErrorHelper.CreateException(400, "Input error");
            Assert.IsType<InputException>(ex);
            Assert.Equal(400, ex.ErrorCode);
        }

        [Fact]
        public void ErrorHelper_CreateException_System()
        {
            var ex = ErrorHelper.CreateException(500, "System error");
            Assert.IsType<Core.SystemException>(ex);
            Assert.Equal(500, ex.ErrorCode);
        }

        [Fact]
        public void ErrorHelper_CreateException_Internal()
        {
            var ex = ErrorHelper.CreateException(900, "Internal error");
            Assert.IsType<InternalException>(ex);
            Assert.Equal(900, ex.ErrorCode);
        }

        [Fact]
        public void ErrorHelper_CreateException_Unknown()
        {
            var ex = ErrorHelper.CreateException(600, "Unknown error");
            Assert.IsType<GoudEngineException>(ex);
            Assert.IsNotType<ContextException>(ex);
            Assert.Equal(600, ex.ErrorCode);
        }

        [Fact]
        public void ErrorHelper_ThrowIfFailed_Success()
        {
            // Should not throw
            ErrorHelper.ThrowIfFailed(true, 0, "Test operation");
        }

        [Fact]
        public void ErrorHelper_ThrowIfFailed_Failure()
        {
            var ex = Assert.Throws<ResourceException>(() =>
                ErrorHelper.ThrowIfFailed(false, 100, "Load texture"));

            Assert.Equal(100, ex.ErrorCode);
            Assert.Contains("Load texture", ex.Message);
            Assert.Contains("failed", ex.Message);
        }

        [Fact]
        public void ErrorHelper_ValidateEntityId_Valid()
        {
            var entityId = new GoudEntityId(12345);
            // Should not throw
            ErrorHelper.ValidateEntityId(entityId, "entityId");
        }

        [Fact]
        public void ErrorHelper_ValidateEntityId_Invalid()
        {
            var entityId = GoudEntityId.Invalid;

            var ex = Assert.Throws<ArgumentException>(() =>
                ErrorHelper.ValidateEntityId(entityId, "entityId"));

            Assert.Contains("entityId", ex.Message);
            Assert.Contains("invalid", ex.Message.ToLower());
        }

        [Fact]
        public void ErrorHelper_ValidateContextId_Valid()
        {
            var contextId = new GoudContextId(12345);
            // Should not throw
            ErrorHelper.ValidateContextId(contextId, "contextId");
        }

        [Fact]
        public void ErrorHelper_ValidateContextId_Invalid()
        {
            var contextId = GoudContextId.Invalid;

            var ex = Assert.Throws<ArgumentException>(() =>
                ErrorHelper.ValidateContextId(contextId, "contextId"));

            Assert.Contains("contextId", ex.Message);
            Assert.Contains("invalid", ex.Message.ToLower());
        }

        [Fact]
        public void ErrorHelper_ValidateHandle_Valid()
        {
            // Should not throw
            ErrorHelper.ValidateHandle(12345, "handle");
        }

        [Fact]
        public void ErrorHelper_ValidateHandle_Zero()
        {
            var ex = Assert.Throws<ArgumentException>(() =>
                ErrorHelper.ValidateHandle(0, "handle"));

            Assert.Contains("handle", ex.Message);
            Assert.Contains("invalid", ex.Message.ToLower());
        }

        [Fact]
        public void ErrorHelper_ValidateHandle_MaxValue()
        {
            var ex = Assert.Throws<ArgumentException>(() =>
                ErrorHelper.ValidateHandle(ulong.MaxValue, "handle"));

            Assert.Contains("handle", ex.Message);
            Assert.Contains("invalid", ex.Message.ToLower());
        }

        // ====================================================================
        // ErrorExtensions Tests
        // ====================================================================

        [Fact]
        public void ErrorExtensions_TryExecute_Success()
        {
            int result;
            Exception? error;

            bool success = ErrorExtensions.TryExecute(() => 42, out result, out error);

            Assert.True(success);
            Assert.Equal(42, result);
            Assert.Null(error);
        }

        [Fact]
        public void ErrorExtensions_TryExecute_Failure()
        {
            int result;
            Exception? error;

            bool success = ErrorExtensions.TryExecute<int>(() =>
            {
                throw new InvalidOperationException("Test error");
            }, out result, out error);

            Assert.False(success);
            Assert.Equal(0, result);
            Assert.NotNull(error);
            Assert.IsType<InvalidOperationException>(error);
            Assert.Equal("Test error", error.Message);
        }

        [Fact]
        public void ErrorExtensions_TryExecute_Action_Success()
        {
            Exception? error;
            int counter = 0;

            bool success = ErrorExtensions.TryExecute(() =>
            {
                counter = 42;
            }, out error);

            Assert.True(success);
            Assert.Equal(42, counter);
            Assert.Null(error);
        }

        [Fact]
        public void ErrorExtensions_TryExecute_Action_Failure()
        {
            Exception? error;

            bool success = ErrorExtensions.TryExecute(() =>
            {
                throw new InvalidOperationException("Test error");
            }, out error);

            Assert.False(success);
            Assert.NotNull(error);
            Assert.IsType<InvalidOperationException>(error);
        }

        // ====================================================================
        // ErrorCategory Tests
        // ====================================================================

        [Fact]
        public void ErrorCategory_EnumValues()
        {
            Assert.Equal(0, (int)ErrorCategory.Success);
            Assert.Equal(1, (int)ErrorCategory.Context);
            Assert.Equal(100, (int)ErrorCategory.Resource);
            Assert.Equal(200, (int)ErrorCategory.Graphics);
            Assert.Equal(300, (int)ErrorCategory.Entity);
            Assert.Equal(400, (int)ErrorCategory.Input);
            Assert.Equal(500, (int)ErrorCategory.System);
            Assert.Equal(900, (int)ErrorCategory.Internal);
            Assert.Equal(-1, (int)ErrorCategory.Unknown);
        }
    }
}

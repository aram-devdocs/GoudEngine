using System;
using CsBindgen;

namespace GoudEngine.Core
{
    /// <summary>
    /// Provides rendering functions for a GoudWindow.
    /// </summary>
    public sealed class GoudRenderer
    {
        private readonly CsBindgen.GoudContextId _contextId;

        /// <summary>
        /// Creates a renderer for the specified window.
        /// </summary>
        public GoudRenderer(GoudWindow window)
        {
            if (window == null) throw new ArgumentNullException(nameof(window));
            _contextId = window.ContextId;
        }

        /// <summary>
        /// Begins a new rendering frame.
        /// Call this after PollEvents and before drawing operations.
        /// </summary>
        public bool Begin()
        {
            return NativeMethods.goud_renderer_begin(_contextId);
        }

        /// <summary>
        /// Ends the current rendering frame.
        /// Call this after all drawing operations and before SwapBuffers.
        /// </summary>
        public bool End()
        {
            return NativeMethods.goud_renderer_end(_contextId);
        }

        /// <summary>
        /// Sets the rendering viewport.
        /// </summary>
        public void SetViewport(int x, int y, uint width, uint height)
        {
            NativeMethods.goud_renderer_set_viewport(_contextId, x, y, width, height);
        }

        /// <summary>
        /// Enables alpha blending for transparent sprites.
        /// </summary>
        public void EnableBlending()
        {
            NativeMethods.goud_renderer_enable_blending(_contextId);
        }

        /// <summary>
        /// Disables alpha blending.
        /// </summary>
        public void DisableBlending()
        {
            NativeMethods.goud_renderer_disable_blending(_contextId);
        }

        /// <summary>
        /// Enables depth testing for 3D rendering.
        /// </summary>
        public void EnableDepthTest()
        {
            NativeMethods.goud_renderer_enable_depth_test(_contextId);
        }

        /// <summary>
        /// Disables depth testing.
        /// </summary>
        public void DisableDepthTest()
        {
            NativeMethods.goud_renderer_disable_depth_test(_contextId);
        }

        /// <summary>
        /// Clears the depth buffer.
        /// </summary>
        public void ClearDepth()
        {
            NativeMethods.goud_renderer_clear_depth(_contextId);
        }

        // ====================================================================
        // Textures
        // ====================================================================

        /// <summary>
        /// Loads a texture from a file.
        /// </summary>
        /// <param name="path">Path to the image file.</param>
        /// <returns>Texture handle, or ulong.MaxValue on failure.</returns>
        public unsafe ulong LoadTexture(string path)
        {
            if (string.IsNullOrEmpty(path)) return ulong.MaxValue;
            
            var bytes = System.Text.Encoding.UTF8.GetBytes(path + "\0");
            fixed (byte* pathPtr = bytes)
            {
                return NativeMethods.goud_texture_load(_contextId, pathPtr);
            }
        }

        /// <summary>
        /// Destroys a texture and releases GPU resources.
        /// </summary>
        /// <param name="textureHandle">The texture handle to destroy.</param>
        /// <returns>True if destroyed successfully.</returns>
        public bool DestroyTexture(ulong textureHandle)
        {
            return NativeMethods.goud_texture_destroy(_contextId, textureHandle);
        }
    }

    /// <summary>
    /// Static collision detection utilities.
    /// </summary>
    public static class GoudCollision
    {
        /// <summary>
        /// Collision contact information.
        /// </summary>
        public struct Contact
        {
            public float PointX, PointY;
            public float NormalX, NormalY;
            public float Penetration;
        }

        /// <summary>
        /// Checks collision between two axis-aligned bounding boxes.
        /// </summary>
        public static unsafe bool CheckAABB(
            float centerAX, float centerAY, float halfWidthA, float halfHeightA,
            float centerBX, float centerBY, float halfWidthB, float halfHeightB,
            out Contact contact)
        {
            var nativeContact = new GoudContact();
            bool result = NativeMethods.goud_collision_aabb_aabb(
                centerAX, centerAY, halfWidthA, halfHeightA,
                centerBX, centerBY, halfWidthB, halfHeightB,
                &nativeContact);
            
            contact = new Contact
            {
                PointX = nativeContact.point_x,
                PointY = nativeContact.point_y,
                NormalX = nativeContact.normal_x,
                NormalY = nativeContact.normal_y,
                Penetration = nativeContact.penetration
            };
            
            return result;
        }

        /// <summary>
        /// Simple AABB overlap check without contact info.
        /// </summary>
        public static bool CheckAABBOverlap(
            float minAX, float minAY, float maxAX, float maxAY,
            float minBX, float minBY, float maxBX, float maxBY)
        {
            return NativeMethods.goud_collision_aabb_overlap(
                minAX, minAY, maxAX, maxAY,
                minBX, minBY, maxBX, maxBY);
        }

        /// <summary>
        /// Checks collision between two circles.
        /// </summary>
        public static unsafe bool CheckCircle(
            float centerAX, float centerAY, float radiusA,
            float centerBX, float centerBY, float radiusB,
            out Contact contact)
        {
            var nativeContact = new GoudContact();
            bool result = NativeMethods.goud_collision_circle_circle(
                centerAX, centerAY, radiusA,
                centerBX, centerBY, radiusB,
                &nativeContact);
            
            contact = new Contact
            {
                PointX = nativeContact.point_x,
                PointY = nativeContact.point_y,
                NormalX = nativeContact.normal_x,
                NormalY = nativeContact.normal_y,
                Penetration = nativeContact.penetration
            };
            
            return result;
        }

        /// <summary>
        /// Simple circle overlap check without contact info.
        /// </summary>
        public static bool CheckCircleOverlap(
            float x1, float y1, float r1,
            float x2, float y2, float r2)
        {
            return NativeMethods.goud_collision_circle_overlap(x1, y1, r1, x2, y2, r2);
        }

        /// <summary>
        /// Checks if a point is inside a rectangle.
        /// </summary>
        public static bool PointInRect(float px, float py, float rx, float ry, float rw, float rh)
        {
            return NativeMethods.goud_collision_point_in_rect(px, py, rx, ry, rw, rh);
        }

        /// <summary>
        /// Checks if a point is inside a circle.
        /// </summary>
        public static bool PointInCircle(float px, float py, float cx, float cy, float radius)
        {
            return NativeMethods.goud_collision_point_in_circle(px, py, cx, cy, radius);
        }

        /// <summary>
        /// Returns the distance between two points.
        /// </summary>
        public static float Distance(float x1, float y1, float x2, float y2)
        {
            return NativeMethods.goud_collision_distance(x1, y1, x2, y2);
        }

        /// <summary>
        /// Returns the squared distance between two points (faster, no sqrt).
        /// </summary>
        public static float DistanceSquared(float x1, float y1, float x2, float y2)
        {
            return NativeMethods.goud_collision_distance_squared(x1, y1, x2, y2);
        }
    }
}

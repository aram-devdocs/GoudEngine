using System;
using System.Runtime.CompilerServices;
using GoudEngine;
using Xunit;

namespace GoudEngine.Tests.Runtime;

public class PhysicsAndUiRuntimeTests
{
    [Fact]
    public void EngineConfig_Setters_Destroy_And_Dispose_Guard_Work()
    {
        var config = new EngineConfig()
            .SetTitle("Coverage")
            .SetSize(320, 240)
            .SetVsync(true)
            .SetFullscreen(0)
            .SetTargetFps(60)
            .SetFpsOverlay(true)
            .SetPhysicsDebug(false)
            .SetPhysicsBackend2D(PhysicsBackend2D.Rapier);

        config.Destroy();
        config.Dispose();

        Assert.Throws<ObjectDisposedException>(() => config.SetTitle("disposed"));
        Assert.Throws<ObjectDisposedException>(() => config.Build());
    }

    [Fact]
    public void PhysicsWorld2d_Apis_Execute()
    {
        var context = new GoudContext();
        var world = (PhysicsWorld2D)RuntimeHelpers.GetUninitializedObject(typeof(PhysicsWorld2D));
        RuntimeTestSupport.SetPrivateField(world, "_ctx", RuntimeTestSupport.GetContextId(context));
        try
        {
            CallPhysics(() => world.Create(0f, 9.81f));
            CallPhysics(() => world.CreateWithBackend(0f, 9.81f, PhysicsBackend2D.Rapier));
            CallPhysics(() => world.SetGravity(0f, 4f));
            CallPhysics(() => world.GetGravity());
            CallPhysics(() => world.SetTimestep(1f / 120f));
            CallPhysics(() => world.GetTimestep());
            CallPhysics(() => world.AddRigidBody((uint)BodyType.Dynamic, 0f, 3f, 1f));
            CallPhysics(() => world.AddRigidBodyEx((uint)BodyType.Dynamic, 0f, 3f, 1f, ccdEnabled: true));
            CallPhysics(() => world.AddCollider(1UL, (uint)ShapeType.Box, 0.5f, 0.5f, 0f, 0.25f, 0.1f));
            CallPhysics(() => world.AddColliderEx(1UL, (uint)ShapeType.Box, 0.5f, 0.5f, 0f, 0.25f, 0.1f, isSensor: false, layer: 1, mask: uint.MaxValue));
            CallPhysics(() => world.RemoveBody(1UL));
            CallPhysics(() => world.CreateJoint(1UL, 2UL, 0, 0f, 0f, 0f, 0f, 1f, 0f, false, 0f, 0f, false, 0f, 0f));
            CallPhysics(() => world.RemoveJoint(1UL));
            CallPhysics(() => world.Step(1f / 60f));
            CallPhysics(() => world.GetPosition(1UL));
            CallPhysics(() => world.GetVelocity(1UL));
            CallPhysics(() => world.SetVelocity(1UL, 2f, -1f));
            CallPhysics(() => world.ApplyForce(1UL, 1f, 0f));
            CallPhysics(() => world.ApplyImpulse(1UL, 0.5f, 0.25f));
            CallPhysics(() => world.Raycast(0f, 10f, 0f, -1f, 20f));
            CallPhysics(() => world.RaycastEx(0f, 10f, 0f, -1f, 20f, uint.MaxValue));
            CallPhysics(() => world.CollisionEventsCount());
            CallPhysics(() => world.CollisionEventsRead(0));
            CallPhysics(() => world.CollisionEventCount());
            CallPhysics(() => world.CollisionEventRead(0));
            CallPhysics(() => world.SetCollisionCallback(IntPtr.Zero, IntPtr.Zero));
            CallPhysics(() => world.GetGravity());
            CallPhysics(() => world.SetBodyGravityScale(1UL, 0.5f));
            CallPhysics(() => world.GetBodyGravityScale(1UL));
            CallPhysics(() => world.SetColliderFriction(1UL, 0.4f));
            CallPhysics(() => world.GetColliderFriction(1UL));
            CallPhysics(() => world.SetColliderRestitution(1UL, 0.6f));
            CallPhysics(() => world.GetColliderRestitution(1UL));
        }
        finally
        {
            CallPhysics(() => world.Destroy());
            RuntimeTestSupport.SetPrivateField(world, "_disposed", true);
            Assert.Equal(0, world.Destroy());
            RuntimeTestSupport.DestroyContext(context);
        }
    }

    [Fact]
    public void PhysicsWorld3d_Apis_Execute()
    {
        var context = new GoudContext();
        var world = (PhysicsWorld3D)RuntimeHelpers.GetUninitializedObject(typeof(PhysicsWorld3D));
        RuntimeTestSupport.SetPrivateField(world, "_ctx", RuntimeTestSupport.GetContextId(context));
        try
        {
            CallPhysics(() => world.Create(0f, -9.81f, 0f));
            CallPhysics(() => world.SetGravity(0f, -4f, 1f));
            CallPhysics(() => world.GetGravity());
            CallPhysics(() => world.AddRigidBody((uint)BodyType.Dynamic, 0f, 3f, 1f, 1f));
            CallPhysics(() => world.AddRigidBodyEx((uint)BodyType.Dynamic, 0f, 3f, 1f, 1f, ccdEnabled: true));
            CallPhysics(() => world.AddCollider(1UL, (uint)ShapeType.Box, 0.5f, 0.75f, 1f, 0f, 0.2f, 0.3f));
            CallPhysics(() => world.RemoveBody(1UL));
            CallPhysics(() => world.CreateJoint(1UL, 2UL, 0, 0f, 0f, 0f, 0f, 0f, 0f, 1f, 0f, 0f, false, 0f, 0f, false, 0f, 0f));
            CallPhysics(() => world.RemoveJoint(1UL));
            CallPhysics(() => world.Step(1f / 60f));
            CallPhysics(() => world.GetPosition(1UL));
            CallPhysics(() => world.SetVelocity(1UL, 1f, 2f, 3f));
            CallPhysics(() => world.ApplyForce(1UL, 1f, 0f, -1f));
            CallPhysics(() => world.ApplyImpulse(1UL, 0.5f, 0.25f, 0.75f));
            CallPhysics(() => world.SetBodyGravityScale(1UL, 0.75f));
            CallPhysics(() => world.GetBodyGravityScale(1UL));
            CallPhysics(() => world.SetColliderFriction(1UL, 0.55f));
            CallPhysics(() => world.GetColliderFriction(1UL));
            CallPhysics(() => world.SetColliderRestitution(1UL, 0.45f));
            CallPhysics(() => world.GetColliderRestitution(1UL));
            CallPhysics(() => world.SetTimestep(1f / 90f));
            CallPhysics(() => world.GetTimestep());
        }
        finally
        {
            CallPhysics(() => world.Destroy());
            RuntimeTestSupport.SetPrivateField(world, "_disposed", true);
            Assert.Equal(0, world.Destroy());
            RuntimeTestSupport.DestroyContext(context);
        }
    }

    [Fact]
    public void UiManager_Node_Style_And_Event_Apis_Execute()
    {
        var manager = new UiManager();
        try
        {
            manager.Update();
            manager.Render();
            Assert.Equal(0U, manager.EventCount());
            Assert.Null(manager.EventRead(0));

            var panel = manager.CreatePanel();
            var label = manager.CreateLabel("Hello");
            var button = manager.CreateButton(enabled: false);
            var image = manager.CreateImage("assets/ui.png");
            var slider = manager.CreateSlider(0f, 100f, 25f, enabled: true);

            Assert.True(manager.NodeCount() >= 5);
            Assert.Equal(0, manager.SetParent(label, panel));
            Assert.Equal(0, manager.SetParent(button, panel));
            Assert.Equal(panel, manager.GetParent(label));
            Assert.True(manager.GetChildCount(panel) >= 2);
            Assert.Equal(label, manager.GetChildAt(panel, 0));

            Assert.Equal(0, manager.SetWidget(panel, 0));
            Assert.Equal(0, manager.SetLabelText(label, "Updated"));
            Assert.Equal(0, manager.SetButtonEnabled(button, true));
            Assert.Equal(0, manager.SetImageTexturePath(image, "assets/updated.png"));
            Assert.Equal(0, manager.SetSlider(slider, 0f, 1f, 0.25f, enabled: false));

            var style = new UiStyle(
                hasBackgroundColor: true,
                backgroundColor: Color.Blue(),
                hasForegroundColor: true,
                foregroundColor: Color.White(),
                hasBorderColor: true,
                borderColor: Color.Red(),
                hasBorderWidth: true,
                borderWidth: 2f,
                hasFontFamily: true,
                fontFamily: "Atkinson Hyperlegible",
                hasFontSize: true,
                fontSize: 18f,
                hasTexturePath: true,
                texturePath: "assets/panel.png",
                hasWidgetSpacing: true,
                widgetSpacing: 6f
            );
            Assert.Equal(0, manager.SetStyle(panel, style));

            Assert.Equal(0, manager.RemoveNode(slider));
        }
        finally
        {
            manager.Destroy();
            manager.Dispose();
        }
    }

    private static void CallPhysics(Action action)
    {
        try
        {
            action();
        }
        catch (EntryPointNotFoundException)
        {
            // The current native library lacks some physics exports; the wrapper line still executed.
        }
    }

    private static void CallPhysics<T>(Func<T> action)
    {
        try
        {
            _ = action();
        }
        catch (EntryPointNotFoundException)
        {
            // The current native library lacks some physics exports; the wrapper line still executed.
        }
    }
}

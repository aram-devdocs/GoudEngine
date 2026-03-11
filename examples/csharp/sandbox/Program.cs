using System;
using System.Globalization;
using System.IO;
using System.Text.Json;
using GoudEngine;

internal static class Program
{
    private const int WindowWidth = 1280;
    private const int WindowHeight = 720;
    private const float MoveSpeed = 220f;
    private static readonly float SmokeSeconds = ParseSmokeSeconds();

    private sealed class NetworkState : IDisposable
    {
        private readonly GoudContext _context = new();
        private readonly NetworkEndpoint? _endpoint;
        private ulong? _knownPeerId;
        private float _heartbeatTimer;

        public string Role { get; private set; } = "offline";
        public int PeerCount { get; private set; }
        public string Detail { get; private set; } = "No network provider";

        public NetworkState(ushort port)
        {
            try
            {
                _endpoint = new NetworkManager(_context).Host(NetworkProtocol.Tcp, port);
                Role = "host";
                Detail = $"Hosting localhost:{port}";
            }
            catch
            {
                try
                {
                    _endpoint = new NetworkManager(_context).Connect(NetworkProtocol.Tcp, "127.0.0.1", port);
                    Role = "client";
                    Detail = $"Connected to localhost:{port}";
                }
                catch (Exception ex)
                {
                    Detail = ex.Message;
                }
            }
        }

        public void Update(float dt)
        {
            if (_endpoint is null)
            {
                return;
            }

            _endpoint.Poll();
            PeerCount = _endpoint.PeerCount();
            var packet = _endpoint.Receive();
            if (packet.HasValue)
            {
                _knownPeerId = packet.Value.PeerId;
                Detail = $"Received {packet.Value.Data.Length} bytes from peer {packet.Value.PeerId}";
            }

            _heartbeatTimer += dt;
            if (_heartbeatTimer < 1f)
            {
                return;
            }

            _heartbeatTimer = 0f;
            var payload = System.Text.Encoding.UTF8.GetBytes($"sandbox:{Role}:{PeerCount}");
            try
            {
                if (_endpoint.DefaultPeerId.HasValue)
                {
                    _endpoint.Send(payload);
                }
                else if (_knownPeerId.HasValue)
                {
                    _endpoint.SendTo(_knownPeerId.Value, payload);
                }
            }
            catch (Exception ex)
            {
                Detail = ex.Message;
            }
        }

        public void Dispose()
        {
            try
            {
                _endpoint?.Disconnect();
            }
            catch
            {
            }
            _context.Dispose();
        }
    }

    private readonly record struct SandboxAssets(
        string Background,
        string Sprite,
        string AccentSprite,
        string Texture3D,
        string Font,
        byte[] Audio,
        ushort Port
    );

    private static void Main()
    {
        var repoRoot = FindRepoRoot(AppContext.BaseDirectory);
        var assets = LoadAssets(repoRoot);

        using var game = new GoudGame(WindowWidth, WindowHeight, "GoudEngine Sandbox - C#");
        using var sceneContext = new GoudContext();
        using var network = new NetworkState(assets.Port);
        using var ui = BuildUi();

        var scene2D = sceneContext.SceneCreate("sandbox-2d");
        var scene3D = sceneContext.SceneCreate("sandbox-3d");
        var sceneHybrid = sceneContext.SceneCreate("sandbox-hybrid");

        var background = (ulong)game.LoadTexture(assets.Background);
        var sprite = (ulong)game.LoadTexture(assets.Sprite);
        var accentSprite = (ulong)game.LoadTexture(assets.AccentSprite);
        var texture3D = (uint)game.LoadTexture(assets.Texture3D);
        var font = (ulong)game.LoadFont(assets.Font);

        var cube = game.CreateCube(texture3D, 1.2f, 1.2f, 1.2f);
        var plane = game.CreatePlane(texture3D, 8f, 8f);
        _ = game.AddLight(0, 4f, 6f, -4f, 0f, -1f, 0f, 1f, 0.95f, 0.8f, 4f, 25f, 0f);
        game.SetObjectPosition(plane, 0f, -1f, 0f);
        game.ConfigureGrid(true, 12f, 12);

        var modeIndex = 0;
        var modes = new[] { "2D", "3D", "Hybrid" };
        var playerX = 250f;
        var playerY = 300f;
        var angle = 0f;
        var audioActivated = false;

        while (!game.ShouldClose())
        {
            var dt = game.DeltaTime > 0 ? game.DeltaTime : 0.016f;
            angle += dt;

            if (game.IsKeyJustPressed(Keys.Escape))
            {
                game.Close();
            }
            if (game.IsKeyJustPressed(Keys.Digit1))
            {
                modeIndex = 0;
            }
            if (game.IsKeyJustPressed(Keys.Digit2))
            {
                modeIndex = 1;
            }
            if (game.IsKeyJustPressed(Keys.Digit3))
            {
                modeIndex = 2;
            }
            if (game.IsKeyPressed(Keys.A) || game.IsKeyPressed(Keys.Left))
            {
                playerX -= MoveSpeed * dt;
            }
            if (game.IsKeyPressed(Keys.D) || game.IsKeyPressed(Keys.Right))
            {
                playerX += MoveSpeed * dt;
            }
            if (game.IsKeyPressed(Keys.W) || game.IsKeyPressed(Keys.Up))
            {
                playerY -= MoveSpeed * dt;
            }
            if (game.IsKeyPressed(Keys.S) || game.IsKeyPressed(Keys.Down))
            {
                playerY += MoveSpeed * dt;
            }
            if (game.IsKeyJustPressed(Keys.Space))
            {
                if (!audioActivated)
                {
                    game.AudioActivate();
                    audioActivated = true;
                }
                game.AudioPlay(assets.Audio);
            }

            var currentMode = modes[modeIndex];
            _ = currentMode switch
            {
                "2D" => sceneContext.SceneSetCurrent(scene2D),
                "3D" => sceneContext.SceneSetCurrent(scene3D),
                _ => sceneContext.SceneSetCurrent(sceneHybrid),
            };

            network.Update(dt);
            var mouse = game.GetMousePosition();

            game.BeginFrame(0.07f, 0.10f, 0.14f, 1f);
            game.DrawSprite(background, WindowWidth / 2f, WindowHeight / 2f, WindowWidth, WindowHeight);
            game.DrawQuad(210f, 110f, 320f, 110f, new Color(0.05f, 0.08f, 0.12f, 0.88f));
            game.DrawQuad(620f, 110f, 560f, 110f, new Color(0.08f, 0.12f, 0.18f, 0.88f));
            game.DrawQuad(620f, 630f, 560f, 120f, new Color(0.05f, 0.08f, 0.12f, 0.90f));
            game.DrawQuad(mouse.X, mouse.Y, 14f, 14f, new Color(0.95f, 0.85f, 0.20f, 0.95f));

            if (currentMode is "2D" or "Hybrid")
            {
                game.DrawQuad(920f, 260f, 180f, 40f, new Color(0.20f, 0.55f, 0.95f, 0.80f));
                game.DrawSprite(sprite, playerX, playerY, 64f, 64f, angle * 0.25f);
                game.DrawSprite(accentSprite, 1040f, 420f, 72f, 240f);
            }

            if (currentMode is "3D" or "Hybrid")
            {
                game.SetCameraPosition3D(0f, 3f, -9.5f);
                game.SetCameraRotation3D(-10f, angle * 20f, 0f);
                game.SetObjectPosition(cube, 0f, 1f + 0.35f * MathF.Sin(angle * 2f), 0f);
                game.SetObjectRotation(cube, 0f, angle * 55f, 0f);
                game.Render3D();
            }

            var caps = game.GetRenderCapabilities();
            var physics = game.GetPhysicsCapabilities();
            var audio = game.GetAudioCapabilities();
            var networkCaps = TryGetNetworkCaps(game);
            var lines = new[]
            {
                "GoudEngine Sandbox",
                $"Mode: {currentMode}  (1/2/3 to switch)",
                "Movement: WASD / Arrows",
                "Audio: SPACE",
                $"Mouse marker: ({mouse.X:0}, {mouse.Y:0})",
                $"Render caps: tex={caps.MaxTextureSize} instancing={caps.SupportsInstancing}",
                $"Physics caps: joints={physics.SupportsJoints} maxBodies={physics.MaxBodies}",
                $"Audio caps: spatial={audio.SupportsSpatial} channels={audio.MaxChannels}",
                $"Scene count: {sceneContext.SceneCount()} current={sceneContext.SceneGetCurrent()}",
                $"UI nodes: {ui.NodeCount()}",
                $"Network role: {network.Role} peers={network.PeerCount}",
                $"Network detail: {network.Detail}",
                $"Network caps: {(networkCaps?.MaxConnections.ToString() ?? "unsupported")}",
            };

            for (var i = 0; i < lines.Length; i++)
            {
                game.DrawText(font, lines[i], 40f, 40f + i * 22f, 18f, TextAlignment.Left, 0f, 1f, TextDirection.Auto, new Color(1f, 1f, 1f, 1f));
            }

            ui.Update();
            ui.Render();
            game.EndFrame();
            if (SmokeSeconds > 0f && angle >= SmokeSeconds)
            {
                game.Close();
            }
        }
    }

    private static float ParseSmokeSeconds()
    {
        var raw = Environment.GetEnvironmentVariable("GOUD_SANDBOX_SMOKE_SECONDS");
        if (string.IsNullOrWhiteSpace(raw))
        {
            return 0f;
        }

        return float.TryParse(raw, NumberStyles.Float, CultureInfo.InvariantCulture, out var value) && value > 0f
            ? value
            : 0f;
    }

    private static SandboxAssets LoadAssets(string repoRoot)
    {
        using var doc = JsonDocument.Parse(File.ReadAllText(Path.Combine(repoRoot, "examples", "shared", "sandbox", "manifest.json")));
        var assets = doc.RootElement.GetProperty("assets");
        return new SandboxAssets(
            Path.Combine(repoRoot, assets.GetProperty("background").GetString()!),
            Path.Combine(repoRoot, assets.GetProperty("sprite").GetString()!),
            Path.Combine(repoRoot, assets.GetProperty("accent_sprite").GetString()!),
            Path.Combine(repoRoot, assets.GetProperty("texture3d").GetString()!),
            Path.Combine(repoRoot, assets.GetProperty("font").GetString()!),
            File.ReadAllBytes(Path.Combine(repoRoot, assets.GetProperty("audio").GetString()!)),
            checked((ushort)doc.RootElement.GetProperty("network_port").GetInt32())
        );
    }

    private static string FindRepoRoot(string startDirectory)
    {
        var current = new DirectoryInfo(startDirectory);
        while (current is not null)
        {
            var cargoToml = Path.Combine(current.FullName, "Cargo.toml");
            var examplesDir = Path.Combine(current.FullName, "examples");
            if (File.Exists(cargoToml) && Directory.Exists(examplesDir))
            {
                return current.FullName;
            }

            current = current.Parent;
        }

        throw new DirectoryNotFoundException($"Could not locate repository root from {startDirectory}");
    }

    private static UiManager BuildUi()
    {
        var ui = new UiManager();
        var root = ui.CreatePanel();
        var title = ui.CreateLabel("Sandbox Widgets");
        var button = ui.CreateButton(true);
        ui.SetParent(title, root);
        ui.SetParent(button, root);
        ui.SetLabelText(title, "Sandbox Widgets");
        ui.SetButtonEnabled(button, true);
        return ui;
    }

    private static NetworkCapabilities? TryGetNetworkCaps(GoudGame game)
    {
        try
        {
            return game.GetNetworkCapabilities();
        }
        catch
        {
            return null;
        }
    }
}

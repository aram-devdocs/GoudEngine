using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Text.Json;
using System.Text;
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
        public string Label { get; private set; } = "solo";
        public int PeerCount { get; private set; }
        public string Detail { get; private set; } = "No network provider";
        public bool HasRemoteState { get; private set; }
        public float RemoteX { get; private set; }
        public float RemoteY { get; private set; }
        public string RemoteMode { get; private set; } = "2D";
        public string RemoteLabel { get; private set; } = "waiting";

        public NetworkState(ushort port)
        {
            try
            {
                _endpoint = new NetworkManager(_context).Host(NetworkProtocol.Tcp, port);
                Role = "host";
                Label = "waiting";
                Detail = $"Hosting localhost:{port}";
            }
            catch
            {
                try
                {
                    _endpoint = new NetworkManager(_context).Connect(NetworkProtocol.Tcp, "127.0.0.1", port);
                    Role = "client";
                    Label = "connected";
                    Detail = $"Connected to localhost:{port}";
                }
                catch (Exception ex)
                {
                    Detail = ex.Message;
                }
            }
        }

        public void Update(float dt, float playerX, float playerY, string mode, string packetVersion)
        {
            if (_endpoint is null)
            {
                return;
            }

            _endpoint.Poll();
            PeerCount = _endpoint.PeerCount();
            Label = Role switch
            {
                "host" when PeerCount <= 0 => "waiting",
                "host" => "connected",
                "client" => "connected",
                _ => "solo",
            };
            var packet = _endpoint.Receive();
            if (packet.HasValue)
            {
                _knownPeerId = packet.Value.PeerId;
                ParseRemoteState(packet.Value.Data);
                Detail = $"Peer {packet.Value.PeerId} synced in {RemoteMode} ({RemoteLabel})";
            }

            _heartbeatTimer += dt;
            if (_heartbeatTimer < 1f)
            {
                return;
            }

            _heartbeatTimer = 0f;
            var payload = Encoding.UTF8.GetBytes(
                string.Create(
                    CultureInfo.InvariantCulture,
                    $"sandbox|{packetVersion}|{Role}|{mode}|{playerX:0.0}|{playerY:0.0}|{Label}"
                )
            );
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

        private void ParseRemoteState(byte[] payload)
        {
            var parts = Encoding.UTF8.GetString(payload).Split('|', StringSplitOptions.RemoveEmptyEntries);
            if (parts.Length != 7 || parts[0] != "sandbox" || parts[1] != "v1")
            {
                return;
            }

            if (float.TryParse(parts[4], NumberStyles.Float, CultureInfo.InvariantCulture, out var x) &&
                float.TryParse(parts[5], NumberStyles.Float, CultureInfo.InvariantCulture, out var y))
            {
                HasRemoteState = true;
                RemoteX = x;
                RemoteY = y;
                RemoteMode = parts[3];
                RemoteLabel = parts[6];
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
        string Title,
        string OverviewTitle,
        string StatusTitle,
        string NextStepsTitle,
        string Background,
        string Sprite,
        string AccentSprite,
        string Texture3D,
        string Font,
        byte[] Audio,
        ushort Port,
        string PacketVersion,
        string Tagline,
        string[] Overview,
        string[] NextSteps,
        string[] StatusRows,
        string[] NextStepDynamicRows,
        SceneEntry[] Scenes,
        string WebNetworkingBlocker,
        string WebRendererBlocker,
        HudLayout Layout,
        HudTypography Typography
    );

    private readonly record struct SceneEntry(string Key, string Mode, string Label);

    private readonly record struct HudRect(float X, float Y, float Width, float Height);

    private readonly record struct OverviewTextLayout(float X, float TitleY, float TaglineY, float MaxWidth);

    private readonly record struct StatusTextLayout(float X, float TitleY, float MaxWidth);

    private readonly record struct NextTextLayout(float X, float TitleY, float MaxWidth);

    private readonly record struct SceneLabelLayout(float X, float Y, float MaxWidth);

    private readonly record struct HudLayout(
        HudRect OverviewPanel,
        HudRect StatusPanel,
        HudRect NextPanel,
        HudRect SceneBadge,
        OverviewTextLayout OverviewText,
        StatusTextLayout StatusText,
        NextTextLayout NextText,
        SceneLabelLayout SceneLabel
    );

    private readonly record struct OverviewLineAdvances(float Title, float Tagline, float Body);

    private readonly record struct StatusLineAdvances(float Title, float Body);

    private readonly record struct NextLineAdvances(float Title, float Body);

    private readonly record struct OverviewTypography(float TitleSize, float TaglineSize, float BodySize, float LineSpacing, OverviewLineAdvances LineAdvances);

    private readonly record struct StatusTypography(float TitleSize, float BodySize, float LineSpacing, StatusLineAdvances LineAdvances);

    private readonly record struct NextTypography(float TitleSize, float BodySize, float LineSpacing, NextLineAdvances LineAdvances);

    private readonly record struct SceneLabelTypography(float Size, float LineSpacing);

    private readonly record struct HudTypography(
        OverviewTypography Overview,
        StatusTypography Status,
        NextTypography Next,
        SceneLabelTypography SceneLabel
    );

    private static void Main()
    {
        var repoRoot = FindRepoRoot(AppContext.BaseDirectory);
        var assets = LoadAssets(repoRoot);

        using var game = new GoudGame(WindowWidth, WindowHeight, $"{assets.Title} - C#");
        using var sceneContext = new GoudContext();
        using var network = new NetworkState(assets.Port);
        using var ui = BuildUi();

        var scene2D = sceneContext.SceneCreate("sandbox-2d");
        var scene3D = sceneContext.SceneCreate("sandbox-3d");
        var sceneHybrid = sceneContext.SceneCreate("sandbox-hybrid");
        _ = sceneContext.SceneSetCurrent(scene2D);

        var background = (ulong)game.LoadTexture(assets.Background);
        var sprite = (ulong)game.LoadTexture(assets.Sprite);
        var accentSprite = (ulong)game.LoadTexture(assets.AccentSprite);
        var font = (ulong)game.LoadFont(assets.Font);

        var cube = 0u;
        var plane = 0u;
        var has3DSetup = false;
        bool Ensure3DSetup()
        {
            if (has3DSetup)
            {
                return true;
            }

            try
            {
                var texture3D = (uint)game.LoadTexture(assets.Texture3D);
                cube = game.CreateCube(texture3D, 1.2f, 1.2f, 1.2f);
                plane = game.CreatePlane(texture3D, 8f, 8f);
                // Key + fill + rim lights keep textured 3D content readable in modes 2/3.
                _ = game.AddLight(0, 4f, 6f, -4f, 0f, -1f, 0f, 1f, 0.95f, 0.80f, 5f, 28f, 0f);
                _ = game.AddLight(0, -3.5f, 3.5f, -2f, 0f, -0.65f, 0.35f, 0.70f, 0.85f, 1f, 2.5f, 18f, 0f);
                _ = game.AddLight(0, 0f, 2.4f, 7f, 0f, -0.25f, -1f, 0.55f, 0.65f, 0.90f, 1.8f, 20f, 0f);
                game.SetObjectPosition(plane, 0f, -1f, 0f);
                game.ConfigureGrid(true, 12f, 12);
                has3DSetup = cube != 0 && plane != 0;
            }
            catch
            {
                has3DSetup = false;
            }

            return has3DSetup;
        }

        var modeIndex = ParseStartMode();
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
            var is3DFamilyMode = currentMode is "3D" or "Hybrid";
            var sceneEntry = CurrentScene(assets, modeIndex, currentMode);

            network.Update(dt, playerX, playerY, currentMode, assets.PacketVersion);
            var mouse = game.GetMousePosition();

            game.BeginFrame(0.07f, 0.10f, 0.14f, 1f);

            if (is3DFamilyMode && Ensure3DSetup())
            {
                game.EnableDepthTest();
                game.SetCameraPosition3D(0f, 2.2f, currentMode == "3D" ? -7.0f : -7.8f);
                game.SetCameraRotation3D(-7f, currentMode == "3D" ? 0f : 8f, 0f);
                game.SetObjectPosition(cube, 0.85f, 1.2f + 0.26f * MathF.Sin(angle * 2f), 2.1f);
                game.SetObjectRotation(cube, 20f, angle * 46f, 0f);
                game.SetObjectPosition(plane, 0f, -1.2f, 2.5f);
                game.Render3D();
                game.DisableDepthTest();
            }

            if (currentMode == "2D")
            {
                game.DrawSprite(background, WindowWidth / 2f, WindowHeight / 2f, WindowWidth, WindowHeight);
                game.DrawSprite(sprite, playerX, playerY, 64f, 64f, angle * 0.25f);
                game.DrawSprite(accentSprite, 1040f, 420f, 72f, 240f);
                game.DrawQuad(920f, 260f, 180f, 40f, new Color(0.20f, 0.55f, 0.95f, 0.80f));
            }

            if (currentMode == "Hybrid")
            {
                game.DrawSprite(background, WindowWidth / 2f, WindowHeight / 2f, WindowWidth, WindowHeight, 0f, new Color(1f, 1f, 1f, 0.26f));
                game.DrawQuad(640f, 360f, 1280f, 720f, new Color(0.08f, 0.17f, 0.24f, 0.10f));
                game.DrawQuad(640f, 654f, 1280f, 132f, new Color(0.03f, 0.10f, 0.12f, 0.18f));
                game.DrawSprite(sprite, playerX, playerY, 72f, 72f, angle * 0.25f);
                game.DrawSprite(accentSprite, 1044f, 420f, 78f, 250f);
                game.DrawQuad(920f, 260f, 180f, 40f, new Color(0.20f, 0.55f, 0.95f, 0.62f));
            }

            var panelAlpha = is3DFamilyMode ? 0.48f : 0.72f;
            var bottomAlpha = is3DFamilyMode ? 0.55f : 0.78f;
            game.DrawQuad(assets.Layout.OverviewPanel.X, assets.Layout.OverviewPanel.Y, assets.Layout.OverviewPanel.Width, assets.Layout.OverviewPanel.Height, new Color(0.05f, 0.08f, 0.12f, panelAlpha));
            game.DrawQuad(assets.Layout.StatusPanel.X, assets.Layout.StatusPanel.Y, assets.Layout.StatusPanel.Width, assets.Layout.StatusPanel.Height, new Color(0.08f, 0.12f, 0.18f, panelAlpha));
            game.DrawQuad(assets.Layout.NextPanel.X, assets.Layout.NextPanel.Y, assets.Layout.NextPanel.Width, assets.Layout.NextPanel.Height, new Color(0.05f, 0.08f, 0.12f, bottomAlpha));
            game.DrawQuad(assets.Layout.SceneBadge.X, assets.Layout.SceneBadge.Y, assets.Layout.SceneBadge.Width, assets.Layout.SceneBadge.Height, new Color(0.20f, 0.55f, 0.95f, 0.84f));
            game.DrawQuad(mouse.X, mouse.Y, 12f, 12f, new Color(0.95f, 0.85f, 0.20f, 0.95f));

            var caps = game.GetRenderCapabilities();
            var physics = game.GetPhysicsCapabilities();
            var audio = game.GetAudioCapabilities();
            var networkCaps = TryGetNetworkCaps(game);
            var typography = assets.Typography;
            var overviewLines = new List<(string Text, float Size, Color Color, float MaxWidth, float BaseAdvance)>
            {
                (assets.OverviewTitle, typography.Overview.TitleSize, new Color(1f, 1f, 1f, 1f), assets.Layout.OverviewText.MaxWidth, typography.Overview.LineAdvances.Title),
                (assets.Tagline, typography.Overview.TaglineSize, new Color(1f, 1f, 1f, 1f), assets.Layout.OverviewText.MaxWidth, typography.Overview.LineAdvances.Tagline),
            };
            foreach (var line in assets.Overview)
            {
                overviewLines.Add((line, typography.Overview.BodySize, new Color(0.94f, 0.97f, 1f, 1f), assets.Layout.OverviewText.MaxWidth, typography.Overview.LineAdvances.Body));
            }

            var statusLines = new List<(string Text, float Size, Color Color, float MaxWidth, float BaseAdvance)>
            {
                (assets.StatusTitle, typography.Status.TitleSize, new Color(0.95f, 0.90f, 0.40f, 1f), assets.Layout.StatusText.MaxWidth, typography.Status.LineAdvances.Title),
            };
            foreach (var row in assets.StatusRows)
            {
                statusLines.Add((RenderStatusRow(row, assets, sceneEntry, currentMode, mouse, caps, physics, audio, network, networkCaps), typography.Status.BodySize, new Color(0.94f, 0.97f, 1f, 1f), assets.Layout.StatusText.MaxWidth, typography.Status.LineAdvances.Body));
            }

            var nextStepLines = new List<(string Text, float Size, Color Color, float MaxWidth, float BaseAdvance)>
            {
                (assets.NextStepsTitle, typography.Next.TitleSize, new Color(0.95f, 0.90f, 0.40f, 1f), assets.Layout.NextText.MaxWidth, typography.Next.LineAdvances.Title),
            };
            foreach (var line in assets.NextSteps)
            {
                nextStepLines.Add((line, typography.Next.BodySize, new Color(0.94f, 0.97f, 1f, 1f), assets.Layout.NextText.MaxWidth, typography.Next.LineAdvances.Body));
            }
            foreach (var row in assets.NextStepDynamicRows)
            {
                nextStepLines.Add((RenderNextStepRow(row, network, audioActivated), typography.Next.BodySize, new Color(0.94f, 0.97f, 1f, 1f), assets.Layout.NextText.MaxWidth, typography.Next.LineAdvances.Body));
            }

            var overviewY = assets.Layout.OverviewText.TitleY;
            foreach (var line in overviewLines)
            {
                game.DrawText(font, line.Text, assets.Layout.OverviewText.X, overviewY, line.Size, TextAlignment.Left, line.MaxWidth, typography.Overview.LineSpacing, TextDirection.Auto, line.Color);
                overviewY += EffectiveAdvance(line.Text, line.Size, line.MaxWidth, line.BaseAdvance);
            }

            var statusY = assets.Layout.StatusText.TitleY;
            foreach (var line in statusLines)
            {
                game.DrawText(font, line.Text, assets.Layout.StatusText.X, statusY, line.Size, TextAlignment.Left, line.MaxWidth, typography.Status.LineSpacing, TextDirection.Auto, line.Color);
                statusY += EffectiveAdvance(line.Text, line.Size, line.MaxWidth, line.BaseAdvance);
            }

            var nextY = assets.Layout.NextText.TitleY;
            foreach (var line in nextStepLines)
            {
                game.DrawText(font, line.Text, assets.Layout.NextText.X, nextY, line.Size, TextAlignment.Left, line.MaxWidth, typography.Next.LineSpacing, TextDirection.Auto, line.Color);
                nextY += EffectiveAdvance(line.Text, line.Size, line.MaxWidth, line.BaseAdvance);
            }
            game.DrawText(font, sceneEntry.Label, assets.Layout.SceneLabel.X, assets.Layout.SceneLabel.Y, typography.SceneLabel.Size, TextAlignment.Left, assets.Layout.SceneLabel.MaxWidth, typography.SceneLabel.LineSpacing, TextDirection.Auto, new Color(1f, 1f, 1f, 1f));
            if (currentMode is "2D" or "Hybrid" && network.HasRemoteState)
            {
                game.DrawQuad(network.RemoteX, network.RemoteY - 50f, 84f, 18f, new Color(0.96f, 0.70f, 0.20f, 0.92f));
                game.DrawText(font, $"Peer {network.RemoteMode}", network.RemoteX - 32f, network.RemoteY - 56f, 13f, TextAlignment.Left, 0f, 1f, TextDirection.Auto, new Color(0.04f, 0.05f, 0.08f, 1f));
                game.DrawSprite(sprite, network.RemoteX, network.RemoteY, 52f, 52f, -angle * 0.18f);
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

    private static int ParseStartMode()
    {
        var raw = Environment.GetEnvironmentVariable("GOUD_SANDBOX_START_MODE");
        if (string.IsNullOrWhiteSpace(raw))
        {
            return 0;
        }

        if (int.TryParse(raw, NumberStyles.Integer, CultureInfo.InvariantCulture, out var numeric))
        {
            return numeric switch
            {
                1 => 0,
                2 => 1,
                3 => 2,
                _ => 0,
            };
        }

        return raw.Trim().ToLowerInvariant() switch
        {
            "2d" => 0,
            "3d" => 1,
            "hybrid" => 2,
            _ => 0,
        };
    }

    private static float EffectiveAdvance(string text, float fontSize, float maxWidth, float baseAdvance)
    {
        return baseAdvance * EstimateWrappedLineCount(text, fontSize, maxWidth);
    }

    private static int EstimateWrappedLineCount(string text, float fontSize, float maxWidth)
    {
        if (string.IsNullOrWhiteSpace(text) || maxWidth <= 0f)
        {
            return 1;
        }

        var approxGlyphWidth = MathF.Max(fontSize * 0.52f, 1f);
        var maxChars = Math.Max(1, (int)MathF.Floor(maxWidth / approxGlyphWidth));
        var total = 0;
        foreach (var rawLine in text.Split('\n'))
        {
            var words = rawLine.Split(' ', StringSplitOptions.RemoveEmptyEntries);
            if (words.Length == 0)
            {
                total += 1;
                continue;
            }

            var wrapped = 1;
            var current = 0;
            foreach (var word in words)
            {
                var length = word.Length;
                if (current == 0)
                {
                    current = length;
                    continue;
                }

                if (current + 1 + length <= maxChars)
                {
                    current += 1 + length;
                }
                else
                {
                    wrapped += 1;
                    current = length;
                }
            }

            total += wrapped;
        }

        return Math.Max(1, total);
    }

    private static SandboxAssets LoadAssets(string repoRoot)
    {
        using var doc = JsonDocument.Parse(File.ReadAllText(Path.Combine(repoRoot, "examples", "shared", "sandbox", "manifest.json")));
        var root = doc.RootElement;
        var assets = root.GetProperty("assets");
        var hud = root.GetProperty("hud");
        using var contract = JsonDocument.Parse(File.ReadAllText(Path.Combine(repoRoot, "examples", "shared", "sandbox", "contract.json")));
        var contractRoot = contract.RootElement;
        var layoutRoot = contractRoot.GetProperty("layout");
        var typographyRoot = contractRoot.GetProperty("typography");
        var scenes = ReadScenes(root.GetProperty("scenes"));
        return new SandboxAssets(
            root.GetProperty("title").GetString() ?? "GoudEngine Sandbox",
            hud.GetProperty("overview_title").GetString() ?? "Overview",
            hud.GetProperty("status_title").GetString() ?? "Live status",
            hud.GetProperty("next_steps_title").GetString() ?? "Try this next",
            Path.Combine(repoRoot, assets.GetProperty("background").GetString()!),
            Path.Combine(repoRoot, assets.GetProperty("sprite").GetString()!),
            Path.Combine(repoRoot, assets.GetProperty("accent_sprite").GetString()!),
            Path.Combine(repoRoot, assets.GetProperty("texture3d").GetString()!),
            Path.Combine(repoRoot, assets.GetProperty("font").GetString()!),
            File.ReadAllBytes(Path.Combine(repoRoot, assets.GetProperty("audio").GetString()!)),
            checked((ushort)root.GetProperty("network_port").GetInt32()),
            root.TryGetProperty("network", out var network) && network.TryGetProperty("packet_version", out var version)
                ? version.GetString() ?? "v1"
                : "v1",
            hud.GetProperty("tagline").GetString() ?? string.Empty,
            ReadStringArray(contractRoot.GetProperty("overview_items")),
            ReadStringArray(contractRoot.GetProperty("next_step_items")),
            ReadStringArray(contractRoot.GetProperty("status_rows")),
            ReadStringArray(contractRoot.GetProperty("next_step_dynamic_rows")),
            scenes,
            contractRoot.GetProperty("web_blockers").GetProperty("networking").GetString() ?? string.Empty,
            contractRoot.GetProperty("web_blockers").GetProperty("renderer").GetString() ?? string.Empty,
            ReadHudLayout(layoutRoot),
            ReadHudTypography(typographyRoot)
        );
    }

    private static string RenderStatusRow(
        string row,
        SandboxAssets assets,
        SceneEntry sceneEntry,
        string currentMode,
        Vec2 mouse,
        RenderCapabilities caps,
        PhysicsCapabilities physics,
        AudioCapabilities audio,
        NetworkState network,
        NetworkCapabilities? networkCaps
    )
    {
        return row switch
        {
            "scene" => $"Scene: {sceneEntry.Label} ({sceneEntry.Key} to switch)",
            "mouse" => $"Mouse marker: ({mouse.X:0}, {mouse.Y:0})",
            "render_caps" => $"Render caps: tex={caps.MaxTextureSize} instancing={caps.SupportsInstancing}",
            "physics_caps" => $"Physics caps: joints={physics.SupportsJoints} maxBodies={physics.MaxBodies}",
            "audio_caps" => $"Audio caps: spatial={audio.SupportsSpatial} channels={audio.MaxChannels}",
            "scene_count" => $"Scene count: {assets.Scenes.Length} active mode={currentMode}",
            "target" => "Target: desktop",
            "network_role" => $"Network role: {network.Role} peers={network.PeerCount} label={network.Label}",
            "network_detail" => $"Network detail: {network.Detail}{(networkCaps is null ? string.Empty : $" (cap={networkCaps.Value.MaxConnections})")}",
            _ => row,
        };
    }

    private static string RenderNextStepRow(string row, NetworkState network, bool audioActivated)
    {
        return row switch
        {
            "audio_status" => $"Audio status: {(audioActivated ? "active" : "press SPACE to activate")}",
            "network_probe" => network.HasRemoteState
                ? $"Peer sprite live at ({network.RemoteX:0}, {network.RemoteY:0})"
                : "Networking: open a second native sandbox to confirm peer sync.",
            _ => row,
        };
    }

    private static SceneEntry[] ReadScenes(JsonElement scenes)
    {
        var entries = new SceneEntry[scenes.GetArrayLength()];
        var index = 0;
        foreach (var scene in scenes.EnumerateArray())
        {
            entries[index++] = new SceneEntry(
                scene.GetProperty("key").GetString() ?? string.Empty,
                scene.GetProperty("mode").GetString() ?? string.Empty,
                scene.GetProperty("label").GetString() ?? string.Empty
            );
        }
        return entries;
    }

    private static SceneEntry CurrentScene(SandboxAssets assets, int modeIndex, string currentMode)
    {
        if ((uint)modeIndex < (uint)assets.Scenes.Length)
        {
            return assets.Scenes[modeIndex];
        }

        foreach (var scene in assets.Scenes)
        {
            if (string.Equals(scene.Mode, currentMode, StringComparison.Ordinal))
            {
                return scene;
            }
        }

        return new SceneEntry("?", currentMode, $"{currentMode} scene");
    }

    private static string[] ReadStringArray(JsonElement array)
    {
        var items = new string[array.GetArrayLength()];
        var index = 0;
        foreach (var item in array.EnumerateArray())
        {
            items[index++] = item.GetString() ?? string.Empty;
        }
        return items;
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

    private static HudLayout ReadHudLayout(JsonElement layoutRoot)
    {
        return new HudLayout(
            ReadRect(layoutRoot.GetProperty("overview_panel")),
            ReadRect(layoutRoot.GetProperty("status_panel")),
            ReadRect(layoutRoot.GetProperty("next_panel")),
            ReadRect(layoutRoot.GetProperty("scene_badge")),
            ReadOverviewText(layoutRoot.GetProperty("overview_text")),
            ReadStatusText(layoutRoot.GetProperty("status_text")),
            ReadNextText(layoutRoot.GetProperty("next_text")),
            ReadSceneLabel(layoutRoot.GetProperty("scene_label"))
        );
    }

    private static HudRect ReadRect(JsonElement element)
    {
        return new HudRect(
            (float)element.GetProperty("x").GetDouble(),
            (float)element.GetProperty("y").GetDouble(),
            (float)element.GetProperty("width").GetDouble(),
            (float)element.GetProperty("height").GetDouble()
        );
    }

    private static OverviewTextLayout ReadOverviewText(JsonElement element)
    {
        return new OverviewTextLayout(
            (float)element.GetProperty("x").GetDouble(),
            (float)element.GetProperty("title_y").GetDouble(),
            (float)element.GetProperty("tagline_y").GetDouble(),
            (float)element.GetProperty("max_width").GetDouble()
        );
    }

    private static StatusTextLayout ReadStatusText(JsonElement element)
    {
        return new StatusTextLayout(
            (float)element.GetProperty("x").GetDouble(),
            (float)element.GetProperty("title_y").GetDouble(),
            (float)element.GetProperty("max_width").GetDouble()
        );
    }

    private static NextTextLayout ReadNextText(JsonElement element)
    {
        return new NextTextLayout(
            (float)element.GetProperty("x").GetDouble(),
            (float)element.GetProperty("title_y").GetDouble(),
            (float)element.GetProperty("max_width").GetDouble()
        );
    }

    private static SceneLabelLayout ReadSceneLabel(JsonElement element)
    {
        return new SceneLabelLayout(
            (float)element.GetProperty("x").GetDouble(),
            (float)element.GetProperty("y").GetDouble(),
            (float)element.GetProperty("max_width").GetDouble()
        );
    }

    private static HudTypography ReadHudTypography(JsonElement typographyRoot)
    {
        return new HudTypography(
            ReadOverviewTypography(typographyRoot.GetProperty("overview")),
            ReadStatusTypography(typographyRoot.GetProperty("status")),
            ReadNextTypography(typographyRoot.GetProperty("next")),
            ReadSceneLabelTypography(typographyRoot.GetProperty("scene_label"))
        );
    }

    private static OverviewTypography ReadOverviewTypography(JsonElement element)
    {
        var advances = element.GetProperty("line_advances");
        return new OverviewTypography(
            (float)element.GetProperty("title_size").GetDouble(),
            (float)element.GetProperty("tagline_size").GetDouble(),
            (float)element.GetProperty("body_size").GetDouble(),
            (float)element.GetProperty("line_spacing").GetDouble(),
            new OverviewLineAdvances(
                (float)advances.GetProperty("title").GetDouble(),
                (float)advances.GetProperty("tagline").GetDouble(),
                (float)advances.GetProperty("body").GetDouble()
            )
        );
    }

    private static StatusTypography ReadStatusTypography(JsonElement element)
    {
        var advances = element.GetProperty("line_advances");
        return new StatusTypography(
            (float)element.GetProperty("title_size").GetDouble(),
            (float)element.GetProperty("body_size").GetDouble(),
            (float)element.GetProperty("line_spacing").GetDouble(),
            new StatusLineAdvances(
                (float)advances.GetProperty("title").GetDouble(),
                (float)advances.GetProperty("body").GetDouble()
            )
        );
    }

    private static NextTypography ReadNextTypography(JsonElement element)
    {
        var advances = element.GetProperty("line_advances");
        return new NextTypography(
            (float)element.GetProperty("title_size").GetDouble(),
            (float)element.GetProperty("body_size").GetDouble(),
            (float)element.GetProperty("line_spacing").GetDouble(),
            new NextLineAdvances(
                (float)advances.GetProperty("title").GetDouble(),
                (float)advances.GetProperty("body").GetDouble()
            )
        );
    }

    private static SceneLabelTypography ReadSceneLabelTypography(JsonElement element)
    {
        return new SceneLabelTypography(
            (float)element.GetProperty("size").GetDouble(),
            (float)element.GetProperty("line_spacing").GetDouble()
        );
    }
}

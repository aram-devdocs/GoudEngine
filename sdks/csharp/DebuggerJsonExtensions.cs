using System.Text.Json;

namespace GoudEngine;

public static class DebuggerJsonExtensions
{
    public static JsonDocument ParseDebuggerSnapshot(this GoudGame game) =>
        JsonDocument.Parse(game.GetDebuggerSnapshotJson());

    public static JsonDocument ParseDebuggerManifest(this GoudGame game) =>
        JsonDocument.Parse(game.GetDebuggerManifestJson());

    public static JsonDocument ParseDebuggerSnapshot(this GoudContext context) =>
        JsonDocument.Parse(context.GetDebuggerSnapshotJson());

    public static JsonDocument ParseDebuggerManifest(this GoudContext context) =>
        JsonDocument.Parse(context.GetDebuggerManifestJson());
}

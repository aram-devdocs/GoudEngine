export interface DebuggerSnapshotSource {
  getDebuggerSnapshotJson(): string;
}

export interface DebuggerManifestSource {
  getDebuggerManifestJson(): string;
}

export function parseDebuggerSnapshot(source: DebuggerSnapshotSource): unknown {
  return JSON.parse(source.getDebuggerSnapshotJson());
}

export function parseDebuggerManifest(source: DebuggerManifestSource): unknown {
  return JSON.parse(source.getDebuggerManifestJson());
}

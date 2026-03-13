"""Debugger-specific TypeScript Node interface helpers."""


def append_debugger_context_methods(lines: list[str]) -> None:
    """Append the shared debugger context interface methods."""
    lines.extend(
        [
            "  getDebuggerSnapshotJson(): string;",
            "  getDebuggerManifestJson(): string;",
            "  setDebuggerPaused(paused: boolean): void;",
            "  stepDebugger(kind: number, count: number): void;",
            "  setDebuggerTimeScale(scale: number): void;",
            "  setDebuggerDebugDrawEnabled(enabled: boolean): void;",
            "  injectDebuggerKeyEvent(key: number, pressed: boolean): void;",
            "  injectDebuggerMouseButton(button: number, pressed: boolean): void;",
            "  injectDebuggerMousePosition(position: IVec2): void;",
            "  injectDebuggerScroll(delta: IVec2): void;",
            "  setDebuggerProfilingEnabled(enabled: boolean): void;",
            "  setDebuggerSelectedEntity(entityId: number): void;",
            "  clearDebuggerSelectedEntity(): void;",
            "  getMemorySummary(): IMemorySummary;",
            "  captureDebuggerFrame(): IDebuggerCapture;",
            "  startDebuggerRecording(): void;",
            "  stopDebuggerRecording(): IDebuggerReplayArtifact;",
            "  startDebuggerReplay(recording: Uint8Array): void;",
            "  stopDebuggerReplay(): void;",
            "  getDebuggerReplayStatusJson(): string;",
            "  getDebuggerMetricsTraceJson(): string;",
        ]
    )

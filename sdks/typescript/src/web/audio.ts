/**
 * Web Audio API adapter for GoudEngine.
 *
 * Provides a thin wrapper around `AudioContext` that mirrors the engine's
 * audio API. Sounds are loaded via `fetch()`, decoded into `AudioBuffer`s,
 * and played through gain-controlled source nodes.
 */

export interface AudioHandle {
  readonly id: number;
  readonly name: string;
}

export class WebAudioManager {
  private ctx: AudioContext | null = null;
  private masterGain: GainNode | null = null;
  private buffers = new Map<number, AudioBuffer>();
  private nextId = 1;
  private activeSources = new Map<number, AudioBufferSourceNode>();

  /**
   * Initialises the audio context. Must be called from a user gesture
   * handler (click / keydown) to satisfy browser autoplay policies.
   */
  async init(): Promise<void> {
    if (this.ctx) return;
    this.ctx = new AudioContext();
    this.masterGain = this.ctx.createGain();
    this.masterGain.connect(this.ctx.destination);

    if (this.ctx.state === 'suspended') {
      await this.ctx.resume();
    }
  }

  /** Loads a sound from a URL and returns a handle for playback. */
  async load(url: string, name?: string): Promise<AudioHandle> {
    if (!this.ctx) {
      throw new Error('WebAudioManager not initialised — call init() first');
    }

    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(`Failed to load audio "${url}": HTTP ${response.status}`);
    }

    const arrayBuffer = await response.arrayBuffer();
    const audioBuffer = await this.ctx.decodeAudioData(arrayBuffer);

    const id = this.nextId++;
    this.buffers.set(id, audioBuffer);
    return { id, name: name ?? url };
  }

  /** Plays a previously loaded sound. Returns a playback id for stopping. */
  play(handle: AudioHandle, loop_ = false, volume = 1.0): number {
    if (!this.ctx || !this.masterGain) return -1;

    const buffer = this.buffers.get(handle.id);
    if (!buffer) return -1;

    const source = this.ctx.createBufferSource();
    source.buffer = buffer;
    source.loop = loop_;

    const gain = this.ctx.createGain();
    gain.gain.value = volume;
    source.connect(gain);
    gain.connect(this.masterGain);

    const playbackId = this.nextId++;
    this.activeSources.set(playbackId, source);
    source.onended = () => this.activeSources.delete(playbackId);
    source.start();

    return playbackId;
  }

  /** Stops a playing sound by its playback id. */
  stop(playbackId: number): void {
    const source = this.activeSources.get(playbackId);
    if (source) {
      source.stop();
      this.activeSources.delete(playbackId);
    }
  }

  /** Sets the master volume (0.0 – 1.0). */
  setMasterVolume(volume: number): void {
    if (this.masterGain) {
      this.masterGain.gain.value = Math.max(0, Math.min(1, volume));
    }
  }

  /** Suspends the audio context (e.g. on tab blur). */
  async suspend(): Promise<void> {
    if (this.ctx && this.ctx.state === 'running') {
      await this.ctx.suspend();
    }
  }

  /** Resumes the audio context (e.g. on tab focus). */
  async resume(): Promise<void> {
    if (this.ctx && this.ctx.state === 'suspended') {
      await this.ctx.resume();
    }
  }

  /** Releases all resources. */
  destroy(): void {
    for (const source of this.activeSources.values()) {
      try { source.stop(); } catch { /* already stopped */ }
    }
    this.activeSources.clear();
    this.buffers.clear();
    if (this.ctx) {
      this.ctx.close().catch(() => {});
      this.ctx = null;
      this.masterGain = null;
    }
  }
}

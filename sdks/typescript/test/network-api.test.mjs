import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, '..');
const nodeGeneratedPath = path.join(repoRoot, 'src', 'generated', 'node', 'index.g.ts');
const webGeneratedPath = path.join(repoRoot, 'src', 'generated', 'web', 'index.g.ts');
const nativeGeneratedPath = path.join(repoRoot, 'native', 'src', 'game.g.rs');

describe('Generated network SDK surface', () => {
  it('exposes the requested Node wrapper methods on GoudGame', () => {
    const nodeSrc = fs.readFileSync(nodeGeneratedPath, 'utf8');

    const expectedMethods = [
      'networkHost(protocol: number, port: number): number {',
      'networkConnect(protocol: number, address: string, port: number): number {',
      'networkDisconnect(handle: number): number {',
      'networkSend(handle: number, peerId: number, data: Uint8Array, channel: number): number {',
      'networkReceive(handle: number): Uint8Array {',
      'networkPoll(handle: number): number {',
      'getNetworkStats(handle: number): INetworkStats {',
      'networkPeerCount(handle: number): number {',
      'setNetworkSimulation(handle: number, config: INetworkSimulationConfig): number {',
      'clearNetworkSimulation(handle: number): number {',
      'setNetworkOverlayHandle(handle: number): number {',
      'clearNetworkOverlayHandle(): number {',
    ];

    for (const method of expectedMethods) {
      assert.ok(nodeSrc.includes(method), `missing generated Node wrapper method: ${method}`);
    }

    assert.ok(nodeSrc.includes('return this.native.getNetworkStats(handle) as unknown as INetworkStats;'));
    assert.ok(nodeSrc.includes('one_way_latency_ms: config.oneWayLatencyMs,'));
  });

  it('exposes matching native napi methods', () => {
    const nativeSrc = fs.readFileSync(nativeGeneratedPath, 'utf8');

    assert.ok(nativeSrc.includes('pub fn network_host(&self, protocol: i32, port: u16) -> f64 {'));
    assert.ok(nativeSrc.includes('pub fn network_connect(&self, protocol: i32, address: String, port: u16) -> Result<f64> {'));
    assert.ok(nativeSrc.includes('pub fn get_network_stats(&self, handle: f64) -> Result<NapiNetworkStats> {'));
    assert.ok(nativeSrc.includes('goud_network_get_stats_v2 failed with status {}'));
    assert.ok(nativeSrc.includes('pub fn set_network_simulation(&self, handle: f64, config: NapiNetworkSimulationConfig) -> i32 {'));
    assert.ok(nativeSrc.includes('pub fn clear_network_overlay_handle(&self) -> i32 {'));
  });

  it("keeps WASM stubs unsupported with the exact error string", () => {
    const webSrc = fs.readFileSync(webGeneratedPath, 'utf8');

    const expectedStubs = [
      "networkHost(_protocol: number, _port: number): number { throw new Error('Not supported in WASM mode'); }",
      "networkConnect(_protocol: number, _address: string, _port: number): number { throw new Error('Not supported in WASM mode'); }",
      "networkDisconnect(_handle: number): number { throw new Error('Not supported in WASM mode'); }",
      "networkSend(_handle: number, _peerId: number, _data: Uint8Array, _channel: number): number { throw new Error('Not supported in WASM mode'); }",
      "networkReceive(_handle: number): Uint8Array { throw new Error('Not supported in WASM mode'); }",
      "networkPoll(_handle: number): number { throw new Error('Not supported in WASM mode'); }",
      "getNetworkStats(_handle: number): INetworkStats { throw new Error('Not supported in WASM mode'); }",
      "networkPeerCount(_handle: number): number { throw new Error('Not supported in WASM mode'); }",
      "setNetworkSimulation(_handle: number, _config: INetworkSimulationConfig): number { throw new Error('Not supported in WASM mode'); }",
      "clearNetworkSimulation(_handle: number): number { throw new Error('Not supported in WASM mode'); }",
      "setNetworkOverlayHandle(_handle: number): number { throw new Error('Not supported in WASM mode'); }",
      "clearNetworkOverlayHandle(): number { throw new Error('Not supported in WASM mode'); }",
    ];

    for (const stub of expectedStubs) {
      assert.ok(webSrc.includes(stub), `missing generated WASM stub: ${stub}`);
    }
  });
});

import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, '..');
const require = createRequire(import.meta.url);
const nodeGeneratedPath = path.join(repoRoot, 'src', 'generated', 'node', 'index.g.ts');
const webGeneratedPath = path.join(repoRoot, 'src', 'generated', 'web', 'index.g.ts');
const typesGeneratedPath = path.join(repoRoot, 'src', 'generated', 'types', 'engine.g.ts');
const nativeGeneratedPath = path.join(repoRoot, 'native', 'src', 'game.g.rs');
const mainEntryPath = path.join(repoRoot, 'src', 'index.ts');
const nodeEntryPath = path.join(repoRoot, 'src', 'node', 'index.ts');
const webEntryPath = path.join(repoRoot, 'src', 'web', 'index.ts');
const sharedWrapperPath = path.join(repoRoot, 'src', 'shared', 'network.ts');

describe('Generated network SDK surface', () => {
  it('exposes the requested Node wrapper methods on GoudGame', () => {
    const nodeSrc = fs.readFileSync(nodeGeneratedPath, 'utf8');
    const typesSrc = fs.readFileSync(typesGeneratedPath, 'utf8');

    const expectedMethods = [
      'networkHost(protocol: number, port: number): number {',
      'networkConnect(protocol: number, address: string, port: number): number {',
      'networkConnectWithPeer(protocol: number, address: string, port: number): INetworkConnectResult {',
      'networkDisconnect(handle: number): number {',
      'networkSend(handle: number, peerId: number, data: Uint8Array, channel: number): number {',
      'networkReceive(handle: number): Uint8Array {',
      'networkReceivePacket(handle: number): INetworkPacket | null {',
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

    assert.ok(typesSrc.includes('export interface IGoudContext {'));
    assert.ok(typesSrc.includes('export interface INetworkConnectResult { handle: number; peerId: number; }'));
    assert.ok(typesSrc.includes('export interface INetworkPacket { peerId: number; data: Uint8Array; }'));
    assert.ok(nodeSrc.includes('export class GoudContext implements IGoudContext {'));
    assert.ok(nodeSrc.includes('networkConnectWithPeer(protocol: number, address: string, port: number): INetworkConnectResult {'));
    assert.ok(nodeSrc.includes('networkReceivePacket(handle: number): INetworkPacket | null {'));
    assert.ok(nodeSrc.includes('return this.native.getNetworkStats(handle) as unknown as INetworkStats;'));
    assert.ok(nodeSrc.includes('return this.native.networkConnectWithPeer(protocol, address, port) as unknown as INetworkConnectResult;'));
    assert.ok(nodeSrc.includes('return this.native.networkReceivePacket(handle) as unknown as INetworkPacket | null;'));
    assert.ok(nodeSrc.includes('one_way_latency_ms: config.oneWayLatencyMs,'));
  });

  it('exposes matching native napi methods', () => {
    const nativeSrc = fs.readFileSync(nativeGeneratedPath, 'utf8');
    const normalizedNativeSrc = nativeSrc.replace(/\s+/g, ' ');

    assert.ok(nativeSrc.includes('pub struct NapiNetworkConnectResult {'));
    assert.ok(nativeSrc.includes('pub struct NapiNetworkPacket {'));
    assert.ok(nativeSrc.includes('pub struct GoudContext {'));
    assert.ok(nativeSrc.includes('pub fn network_host(&self, protocol: i32, port: u16) -> f64 {'));
    assert.ok(nativeSrc.includes('pub fn network_connect(&self, protocol: i32, address: String, port: u16) -> Result<f64> {'));
    const networkConnectWithPeerSignatureStart = normalizedNativeSrc.indexOf('pub fn network_connect_with_peer(');
    assert.ok(networkConnectWithPeerSignatureStart >= 0);
    const signatureChunk = normalizedNativeSrc.slice(networkConnectWithPeerSignatureStart, networkConnectWithPeerSignatureStart + 180);
    assert.ok(
      /protocol:\s*i32,\s*address:\s*String,\s*port:\s*u16,?\s*\)\s*->\s*Result<NapiNetworkConnectResult>\s*\{/.test(signatureChunk),
    );
    assert.ok(nativeSrc.includes('pub fn network_receive_packet(&self, handle: f64) -> Result<Option<NapiNetworkPacket>> {'));
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
      "networkConnectWithPeer(_protocol: number, _address: string, _port: number): INetworkConnectResult { throw new Error('Not supported in WASM mode'); }",
      "networkDisconnect(_handle: number): number { throw new Error('Not supported in WASM mode'); }",
      "networkSend(_handle: number, _peerId: number, _data: Uint8Array, _channel: number): number { throw new Error('Not supported in WASM mode'); }",
      "networkReceive(_handle: number): Uint8Array { throw new Error('Not supported in WASM mode'); }",
      "networkReceivePacket(_handle: number): INetworkPacket | null { throw new Error('Not supported in WASM mode'); }",
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

  it('exports handwritten wrapper entrypoints for default, node, and web builds', () => {
    const mainSrc = fs.readFileSync(mainEntryPath, 'utf8');
    const nodeSrc = fs.readFileSync(nodeEntryPath, 'utf8');
    const webSrc = fs.readFileSync(webEntryPath, 'utf8');
    const sharedSrc = fs.readFileSync(sharedWrapperPath, 'utf8');

    const expectedMain = [
      "export * from './generated/index.g.js';",
      "export { NetworkManager, NetworkEndpoint } from './shared/network.js';",
      "export type { NetworkContextLike } from './shared/network.js';",
      "export { NetworkProtocol } from './generated/types/input.g.js';",
    ];
    for (const line of expectedMain) {
      assert.ok(mainSrc.includes(line), `missing default entrypoint export: ${line}`);
    }

    const expectedNode = [
      "export * from '../generated/node/index.g.js';",
      "export { NetworkManager, NetworkEndpoint } from '../shared/network.js';",
      "export type { NetworkContextLike } from '../shared/network.js';",
      "export { NetworkProtocol } from '../generated/types/input.g.js';",
    ];
    for (const line of expectedNode) {
      assert.ok(nodeSrc.includes(line), `missing node entrypoint export: ${line}`);
    }

    const expectedWeb = [
      "export * from '../generated/web/index.g.js';",
      "export { NetworkManager, NetworkEndpoint } from '../shared/network.js';",
      "export type { NetworkContextLike } from '../shared/network.js';",
      "export { NetworkProtocol } from '../generated/types/input.g.js';",
    ];
    for (const line of expectedWeb) {
      assert.ok(webSrc.includes(line), `missing web entrypoint export: ${line}`);
    }

    const expectedShared = [
      'export interface NetworkContextLike {',
      'export class NetworkManager {',
      'export class NetworkEndpoint {',
      'host(protocol: number, port: number): NetworkEndpoint {',
      'connect(protocol: number, address: string, port: number): NetworkEndpoint {',
      'receive(): INetworkPacket | null {',
      'send(data: Uint8Array, channel = 0): number {',
      'sendTo(peerId: number, data: Uint8Array, channel = 0): number {',
      'poll(): number {',
      'disconnect(): number {',
      'getStats(): INetworkStats {',
      'peerCount(): number {',
      'setSimulation(config: INetworkSimulationConfig): number {',
      'clearSimulation(): number {',
      'setOverlayTarget(): number {',
      'clearOverlayTarget(): number {',
      'Use sendTo(peerId, data, channel) instead.',
    ];
    for (const line of expectedShared) {
      assert.ok(sharedSrc.includes(line), `missing shared wrapper member: ${line}`);
    }
  });

  it('exports network wrappers from the built package entrypoints', () => {
    const mainSdk = require(path.join(repoRoot, 'dist', 'index.js'));
    const nodeSdk = require(path.join(repoRoot, 'dist', 'node', 'index.js'));

    assert.equal(typeof mainSdk.NetworkManager, 'function');
    assert.equal(typeof mainSdk.NetworkEndpoint, 'function');
    assert.equal(typeof mainSdk.NetworkProtocol, 'object');
    assert.equal(typeof nodeSdk.NetworkManager, 'function');
    assert.equal(typeof nodeSdk.NetworkEndpoint, 'function');
    assert.equal(typeof nodeSdk.GoudContext, 'function');
  });
});

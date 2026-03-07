/**
 * Tests for GoudEngine error types.
 * Verifies that error classes are importable, have correct properties,
 * and that GoudError.fromCode() dispatches to the right subclass.
 *
 * Run with: node --test test/errors.test.mjs
 */

import { describe, it } from 'node:test';
import assert from 'node:assert/strict';

import {
  GoudError,
  GoudContextError,
  GoudResourceError,
  GoudGraphicsError,
  GoudEntityError,
  GoudInputError,
  GoudSystemError,
  GoudProviderError,
  GoudInternalError,
  RecoveryClass,
} from '../errors.js';

describe('Error class imports', () => {
  it('all error classes are defined', () => {
    assert.ok(GoudError, 'GoudError should be defined');
    assert.ok(GoudContextError, 'GoudContextError should be defined');
    assert.ok(GoudResourceError, 'GoudResourceError should be defined');
    assert.ok(GoudGraphicsError, 'GoudGraphicsError should be defined');
    assert.ok(GoudEntityError, 'GoudEntityError should be defined');
    assert.ok(GoudInputError, 'GoudInputError should be defined');
    assert.ok(GoudSystemError, 'GoudSystemError should be defined');
    assert.ok(GoudProviderError, 'GoudProviderError should be defined');
    assert.ok(GoudInternalError, 'GoudInternalError should be defined');
    assert.ok(RecoveryClass, 'RecoveryClass should be defined');
  });

  it('RecoveryClass has expected constants', () => {
    assert.equal(RecoveryClass.Recoverable, 0);
    assert.equal(RecoveryClass.Fatal, 1);
    assert.equal(RecoveryClass.Degraded, 2);
  });
});

describe('GoudError base class', () => {
  it('constructs with expected properties', () => {
    const err = new GoudError(1, 'ctx error', 'Context', 'engine', 'init', RecoveryClass.Fatal, 'Call init first');
    assert.equal(err.code, 1);
    assert.equal(err.category, 'Context');
    assert.equal(err.subsystem, 'engine');
    assert.equal(err.operation, 'init');
    assert.equal(err.recovery, RecoveryClass.Fatal);
    assert.equal(err.recoveryHint, 'Call init first');
    assert.equal(err.message, 'ctx error');
    assert.equal(err.name, 'GoudError');
  });

  it('is an instance of Error', () => {
    const err = new GoudError(1, 'msg', 'Context', '', '', 1, '');
    assert.ok(err instanceof Error);
    assert.ok(err instanceof GoudError);
  });
});

describe('GoudError.fromCode() subclass dispatch', () => {
  it('returns GoudContextError for code 1 (range 1-99)', () => {
    const err = GoudError.fromCode(1, 'not initialised');
    assert.ok(err instanceof GoudContextError, `Expected GoudContextError, got ${err.constructor.name}`);
    assert.equal(err.category, 'Context');
    assert.equal(err.code, 1);
  });

  it('returns GoudContextError for code 50 (range 1-99)', () => {
    const err = GoudError.fromCode(50, 'context error');
    assert.ok(err instanceof GoudContextError);
    assert.equal(err.category, 'Context');
  });

  it('returns GoudResourceError for code 100 (range 100-199)', () => {
    const err = GoudError.fromCode(100, 'file not found');
    assert.ok(err instanceof GoudResourceError, `Expected GoudResourceError, got ${err.constructor.name}`);
    assert.equal(err.category, 'Resource');
    assert.equal(err.code, 100);
  });

  it('returns GoudResourceError for code 110 and populates recoveryHint', () => {
    const err = GoudError.fromCode(110, 'invalid handle');
    assert.ok(err instanceof GoudResourceError);
    assert.equal(err.recoveryHint, 'Ensure the handle was obtained from a valid creation call');
  });

  it('returns GoudGraphicsError for code 200 (range 200-299)', () => {
    const err = GoudError.fromCode(200, 'shader compile failed');
    assert.ok(err instanceof GoudGraphicsError, `Expected GoudGraphicsError, got ${err.constructor.name}`);
    assert.equal(err.category, 'Graphics');
  });

  it('returns GoudEntityError for code 300 (range 300-399)', () => {
    const err = GoudError.fromCode(300, 'entity not found');
    assert.ok(err instanceof GoudEntityError, `Expected GoudEntityError, got ${err.constructor.name}`);
    assert.equal(err.category, 'Entity');
  });

  it('returns GoudInputError for code 400 (range 400-499)', () => {
    const err = GoudError.fromCode(400, 'device not found');
    assert.ok(err instanceof GoudInputError, `Expected GoudInputError, got ${err.constructor.name}`);
    assert.equal(err.category, 'Input');
  });

  it('returns GoudSystemError for code 500 (range 500-599)', () => {
    const err = GoudError.fromCode(500, 'window failed');
    assert.ok(err instanceof GoudSystemError, `Expected GoudSystemError, got ${err.constructor.name}`);
    assert.equal(err.category, 'System');
  });

  it('returns GoudProviderError for code 600 (range 600-699)', () => {
    const err = GoudError.fromCode(600, 'provider config failed');
    assert.ok(err instanceof GoudProviderError, `Expected GoudProviderError, got ${err.constructor.name}`);
    assert.equal(err.category, 'Provider');
  });

  it('returns GoudInternalError for code 900 (range 900+)', () => {
    const err = GoudError.fromCode(900, 'engine bug');
    assert.ok(err instanceof GoudInternalError, `Expected GoudInternalError, got ${err.constructor.name}`);
    assert.equal(err.category, 'Internal');
    assert.equal(err.recoveryHint, 'Report the error with full details; this is likely an engine bug');
  });

  it('accepts subsystem and operation context', () => {
    const err = GoudError.fromCode(100, 'not found', 'assets', 'loadTexture');
    assert.equal(err.subsystem, 'assets');
    assert.equal(err.operation, 'loadTexture');
  });

  it('populates recoveryHint for known code 1', () => {
    const err = GoudError.fromCode(1, 'not init');
    assert.equal(err.recoveryHint, 'Call the initialization function first');
  });
});

describe('instanceof checks across subclasses', () => {
  it('GoudContextError is instanceof GoudError', () => {
    const err = GoudError.fromCode(1, 'msg');
    assert.ok(err instanceof GoudError);
    assert.ok(err instanceof GoudContextError);
    assert.ok(!(err instanceof GoudResourceError));
  });

  it('GoudResourceError is instanceof GoudError but not GoudContextError', () => {
    const err = GoudError.fromCode(100, 'msg');
    assert.ok(err instanceof GoudError);
    assert.ok(err instanceof GoudResourceError);
    assert.ok(!(err instanceof GoudContextError));
  });

  it('all subclasses set correct name property', () => {
    const pairs = [
      [GoudError.fromCode(1, ''), 'GoudContextError'],
      [GoudError.fromCode(100, ''), 'GoudResourceError'],
      [GoudError.fromCode(200, ''), 'GoudGraphicsError'],
      [GoudError.fromCode(300, ''), 'GoudEntityError'],
      [GoudError.fromCode(400, ''), 'GoudInputError'],
      [GoudError.fromCode(500, ''), 'GoudSystemError'],
      [GoudError.fromCode(600, ''), 'GoudProviderError'],
      [GoudError.fromCode(900, ''), 'GoudInternalError'],
    ];
    for (const [err, expectedName] of pairs) {
      assert.equal(err.name, expectedName, `Expected name=${expectedName}, got ${err.name}`);
    }
  });
});

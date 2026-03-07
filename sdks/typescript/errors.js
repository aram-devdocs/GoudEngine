// prettier-ignore
/* eslint-disable */
/**
 * Typed error classes for GoudEngine TypeScript SDK.
 * Pure JavaScript - no native binding required.
 */

'use strict';

const RecoveryClass = Object.freeze({ Recoverable: 0, Fatal: 1, Degraded: 2 });

class GoudError extends Error {
  constructor(code, message, category, subsystem, operation, recovery, recoveryHint) {
    super(message);
    this.name = new.target.name;
    this.code = code;
    this.category = category;
    this.subsystem = subsystem;
    this.operation = operation;
    this.recovery = recovery;
    this.recoveryHint = recoveryHint;
    Object.setPrototypeOf(this, new.target.prototype);
  }

  static fromCode(code, message, subsystem = '', operation = '') {
    const category = categoryFromCode(code);
    const recovery = recoveryFromCategory(category);
    const hint = hintFromCode(code);
    const Subclass = CATEGORY_CLASS_MAP[category] ?? GoudError;
    return new Subclass(code, message, category, subsystem, operation, recovery, hint);
  }
}

class GoudContextError extends GoudError {}
class GoudResourceError extends GoudError {}
class GoudGraphicsError extends GoudError {}
class GoudEntityError extends GoudError {}
class GoudInputError extends GoudError {}
class GoudSystemError extends GoudError {}
class GoudProviderError extends GoudError {}
class GoudInternalError extends GoudError {}

const CATEGORY_CLASS_MAP = {
  Context: GoudContextError,
  Resource: GoudResourceError,
  Graphics: GoudGraphicsError,
  Entity: GoudEntityError,
  Input: GoudInputError,
  System: GoudSystemError,
  Provider: GoudProviderError,
  Internal: GoudInternalError,
};

function categoryFromCode(code) {
  if (code >= 900) return 'Internal';
  if (code >= 600) return 'Provider';
  if (code >= 500) return 'System';
  if (code >= 400) return 'Input';
  if (code >= 300) return 'Entity';
  if (code >= 200) return 'Graphics';
  if (code >= 100) return 'Resource';
  if (code >= 1)   return 'Context';
  return 'Unknown';
}

function recoveryFromCategory(category) {
  switch (category) {
    case 'Context':
    case 'Internal':
      return RecoveryClass.Fatal;
    default:
      return RecoveryClass.Recoverable;
  }
}

function hintFromCode(code) {
  return HINTS[code] ?? '';
}

const HINTS = {
  1:   'Call the initialization function first',
  2:   'Shut down the engine before re-initializing',
  3:   'Ensure the context was properly created and not corrupted',
  4:   'Re-initialize the engine to obtain a new context',
  10:  'Check the error message for details and verify dependencies',
  100: 'Verify the file path and check the working directory',
  101: 'Check file permissions and ensure the file is not locked',
  102: 'Verify the file is not corrupted and uses a supported format',
  103: 'Use a unique identifier or remove the existing resource first',
  110: 'Ensure the handle was obtained from a valid creation call',
  111: 'Re-create the resource to get a new handle',
  112: 'Pass the correct handle type for the operation',
  200: 'Review shader source; the error message contains GPU compiler output',
  201: 'Verify shader stage inputs/outputs match and uniforms are declared',
  210: 'Check texture dimensions and format; reduce size or free GPU resources',
  211: 'Reduce buffer size or free unused GPU buffers',
  220: 'Verify attachment formats and dimensions are consistent',
  230: 'Update GPU drivers or select a different supported backend',
  240: 'Verify buffer bindings and shader state; try updating GPU drivers',
  300: 'Verify the entity ID is valid and has not been despawned',
  301: 'Use a different entity ID or remove the existing entity first',
  310: 'Attach the component before accessing it, or check with a has-component query',
  311: 'Use replace/update instead of add, or remove the existing component first',
  320: 'Check for conflicting mutable/immutable access on the same component',
  400: 'Verify the input device is connected and recognized by the OS',
  401: 'Check the action name matches a registered input action',
  500: 'Verify display server is running and window parameters are valid',
  510: 'Check that an audio output device is available',
  520: 'Review physics configuration for invalid values',
  530: 'Check the error message for platform-specific details',
  600: 'Check provider configuration and dependencies',
  601: 'Register the provider before accessing it',
  602: 'Check the error message for operation-specific details',
  900: 'Report the error with full details; this is likely an engine bug',
  901: 'Use an alternative approach or wait for the feature to be implemented',
  902: 'Check the sequence of API calls; the engine may need re-initialization',
};

module.exports = {
  RecoveryClass,
  GoudError,
  GoudContextError,
  GoudResourceError,
  GoudGraphicsError,
  GoudEntityError,
  GoudInputError,
  GoudSystemError,
  GoudProviderError,
  GoudInternalError,
};

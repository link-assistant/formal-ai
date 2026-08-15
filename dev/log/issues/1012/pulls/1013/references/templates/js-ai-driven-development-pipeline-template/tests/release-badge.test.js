/**
 * Tests for npm badge version normalization in release notes.
 * Reproduces issue #40: language-prefixed tags must not be interpolated
 * directly into shields.io static badge URLs.
 */

import { describe, it, expect } from 'test-anywhere';
import {
  buildNpmVersionBadge,
  encodeShieldsStaticBadgeSegment,
  normalizeReleaseVersionForBadge,
} from '../scripts/format-release-notes-helpers.mjs';

describe('release badge version normalization', () => {
  it('strips a plain v prefix for backward compatibility', () => {
    expect(normalizeReleaseVersionForBadge('v1.7.12')).toBe('1.7.12');
  });

  it('strips a js-v prefix from multi-language release tags', () => {
    expect(normalizeReleaseVersionForBadge('js-v1.7.12')).toBe('1.7.12');
  });

  it('strips a js_v prefix from auto-detected multi-language release tags', () => {
    expect(normalizeReleaseVersionForBadge('js_v1.7.12')).toBe('1.7.12');
  });

  it('strips a rust-v prefix from multi-language release tags', () => {
    expect(normalizeReleaseVersionForBadge('rust-v0.3.4')).toBe('0.3.4');
  });

  it('strips a rust_v prefix from multi-language release tags', () => {
    expect(normalizeReleaseVersionForBadge('rust_v0.3.4')).toBe('0.3.4');
  });

  it('escapes hyphens in prerelease versions for shields.io static badge paths', () => {
    expect(encodeShieldsStaticBadgeSegment('1.0.0-alpha.1')).toBe(
      '1.0.0--alpha.1'
    );
  });

  it('builds a valid shields.io badge URL for prefixed tags', () => {
    const badge = buildNpmVersionBadge('my-package', 'js_v1.7.12');

    expect(badge.includes('/badge/npm-1.7.12-blue.svg')).toBe(true);
    expect(badge.includes('/badge/npm-js_v1.7.12-blue.svg')).toBe(false);
    expect(badge.includes('/my-package/v/1.7.12')).toBe(true);
  });

  it('builds a valid shields.io badge URL for prefixed prerelease tags', () => {
    const badge = buildNpmVersionBadge('my-package', 'js_v1.0.0-alpha.1');

    expect(badge.includes('/badge/npm-1.0.0--alpha.1-blue.svg')).toBe(true);
    expect(badge.includes('/my-package/v/1.0.0-alpha.1')).toBe(true);
  });
});

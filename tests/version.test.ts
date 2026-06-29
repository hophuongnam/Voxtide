import { describe, it, expect } from 'vitest';
import { isNewer } from '../src/lib/version';

describe('isNewer', () => {
  it('detects a newer patch', () => expect(isNewer('0.1.8', '0.1.7')).toBe(true));
  it('compares numerically, not lexically (0.1.10 > 0.1.9)', () => expect(isNewer('0.1.10', '0.1.9')).toBe(true));
  it('equal is not newer', () => expect(isNewer('0.1.7', '0.1.7')).toBe(false));
  it('older is not newer', () => expect(isNewer('0.1.6', '0.1.7')).toBe(false));
  it('tolerates a leading v tag', () => expect(isNewer('v0.2.0', '0.1.9')).toBe(true));
  it('minor outranks patch', () => expect(isNewer('0.2.0', '0.1.99')).toBe(true));
});

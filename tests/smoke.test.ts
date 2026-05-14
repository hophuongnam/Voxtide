import { describe, it, expect } from 'vitest';
import { VERSION } from '../src/types';

describe('frontend bundle', () => {
  it('exposes a non-empty VERSION', () => {
    expect(VERSION).toMatch(/^0\.1\./);
  });
});

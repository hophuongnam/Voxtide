import { describe, it, expect, vi } from 'vitest';
// miccapture imports @tauri-apps/api/core (invoke) at module load — stub it so the
// module imports cleanly in jsdom. pickBuiltinMic itself touches none of it.
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
import { pickBuiltinMic } from '../src/lib/miccapture';

const dev = (deviceId: string, label: string, kind = 'audioinput') => ({ deviceId, label, kind });

describe('pickBuiltinMic', () => {
  it('skips a connected Bluetooth headset and returns the built-in mic', () =>
    expect(pickBuiltinMic([dev('default', 'Default'), dev('bt', 'Galaxy Buds Pro'), dev('mic', 'Built-in microphone')])).toBe('mic'));
  it('skips the virtual default/communications aliases (they follow the headset route)', () =>
    expect(pickBuiltinMic([dev('default', 'Default'), dev('communications', 'Communications'), dev('mic', 'phone mic')])).toBe('mic'));
  it('returns undefined when only a headset is present (nothing to pin → falls back to default)', () =>
    expect(pickBuiltinMic([dev('default', 'Default'), dev('bt', 'Sony WH-1000XM5 Hands-Free')])).toBeUndefined());
  it('ignores non-audioinput devices', () =>
    expect(pickBuiltinMic([dev('spk', 'internal speaker', 'audiooutput'), dev('mic', 'internal mic')])).toBe('mic'));
  it('returns undefined for the pre-permission blank list (empty deviceId/label)', () =>
    expect(pickBuiltinMic([dev('', '')])).toBeUndefined());
  // Real Galaxy S24+ enumerateDevices with a "HUAWEI FreeClip" BT headphone connected,
  // verified on-device 2026-06-30: must pick "Speakerphone" (far-field built-in), not the
  // earpiece or the Bluetooth mic. See reference_voxtide_android_webview_audio.
  it('real S24+ list: picks Speakerphone over earpiece + Bluetooth headset', () =>
    expect(pickBuiltinMic([
      dev('default', ''),
      dev('c9d54a-speakerphone', 'Speakerphone'),
      dev('9c1b-earpiece', 'Headset earpiece'),
      dev('50e6-bt', 'Bluetooth headset'),
    ])).toBe('c9d54a-speakerphone'));
});

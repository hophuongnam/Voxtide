import { invoke } from '@tauri-apps/api/core';

// AudioWorklet processor (inlined as a Blob — no separate served file needed).
// Accumulates mono f32 samples and posts ~100 ms batches (1600 samples @ 16 kHz)
// to the main thread.
const WORKLET_SRC = `
class MicCapture extends AudioWorkletProcessor {
  constructor() { super(); this.buf = []; }
  process(inputs) {
    const ch = inputs[0] && inputs[0][0];
    if (ch) {
      for (let i = 0; i < ch.length; i++) this.buf.push(ch[i]);
      if (this.buf.length >= 1600) {
        this.port.postMessage({ samples: this.buf, sampleRate });
        this.buf = [];
      }
    }
    return true;
  }
}
registerProcessor('mic-capture', MicCapture);
`;

let audioCtx: AudioContext | null = null;
let node: AudioWorkletNode | null = null;
let stream: MediaStream | null = null;
let gainNode: GainNode | null = null;

/** Live pipeline vitals for the on-device diagnostic readout. `batches` climbing
 *  proves getUserMedia+worklet+posting work; `sampleRate` must be 16000 or Soniox
 *  gets pitch-shifted audio; `state` must reach 'running' or no audio flows. */
export interface MicStats { state: string; sampleRate: number; batches: number; }

/** Pick the phone's built-in mic from an `enumerateDevices()` list, else undefined.
 *  Skips the virtual `default`/`communications` aliases (they FOLLOW the system
 *  route → a connected Bluetooth/wired headset) and anything whose label looks
 *  external, leaving the on-board mic. Face-to-face mode wants the phone's own mic
 *  (it hears both speakers across the table), never a headset by one person's ear.
 *  Label-based, so undefined just falls back to the system default — no worse than
 *  before. */
export function pickBuiltinMic(devices: { kind: string; deviceId: string; label: string }[]): string | undefined {
  const EXTERNAL = /bluetooth|headset|\bsco\b|hands.?free|wired|airpod|buds|earbud|earphone|headphone/i;
  return devices.find(
    (d) => d.kind === 'audioinput'
      && d.deviceId && d.deviceId !== 'default' && d.deviceId !== 'communications'
      && !EXTERNAL.test(d.label),
  )?.deviceId;
}

/** Start capturing the mic and streaming PCM to Rust. Throws if mic permission
 *  is denied (NotAllowedError) — the caller surfaces that. `onStats` (optional)
 *  reports pipeline vitals for the on-device diagnostic readout. `gain` is the
 *  initial input-gain multiplier (1.0 = unity); change it live via setMicGain.
 *  `agc` enables the browser's automatic gain control (live via setMicAgc). */
export async function startMicCapture(onStats?: (s: MicStats) => void, gain = 1, agc = false): Promise<void> {
  let batches = 0;
  const report = () =>
    onStats?.({ state: audioCtx?.state ?? '—', sampleRate: audioCtx?.sampleRate ?? 0, batches });

  // Far-field capture: noiseSuppression + echoCancellation always OFF (close-talk
  // noise suppression gates far/quiet speech; no playback to echo-cancel on a
  // single shared mic). autoGainControl is user-toggleable (`agc`) — default OFF
  // so the manual mic_gain slider is the primary, predictable level control (AGC
  // auto-rides the level and fights the knob), but some users prefer it on.
  const base: MediaTrackConstraints = { autoGainControl: agc, noiseSuppression: false, echoCancellation: false };
  // Pin the built-in mic so a connected Bluetooth/wired headset can't hijack it —
  // face-to-face needs the on-table phone mic that hears both speakers, not a mic
  // by one person's ear. enumerateDevices() labels are blank until permission is
  // granted, so on first run we open the default to get it, then re-pin below.
  // Enumeration is best-effort: a failure must not break capture (just no pinning).
  const enumerate = async () => {
    try { return await navigator.mediaDevices.enumerateDevices(); } catch { return []; }
  };
  let builtin = pickBuiltinMic(await enumerate());
  try {
    stream = await navigator.mediaDevices.getUserMedia({
      audio: builtin ? { ...base, deviceId: { exact: builtin } } : base,
    });
  } catch {
    stream = await navigator.mediaDevices.getUserMedia({ audio: base });
  }
  if (!builtin) {
    // Permission is granted now → labels are populated. Re-pin if the default
    // device (which a headset hijacks) isn't already the built-in mic.
    builtin = pickBuiltinMic(await enumerate());
    if (builtin && builtin !== stream.getAudioTracks()[0]?.getSettings().deviceId) {
      try {
        const pinned = await navigator.mediaDevices.getUserMedia({ audio: { ...base, deviceId: { exact: builtin } } });
        stream.getTracks().forEach((t) => t.stop());
        stream = pinned;
      } catch { /* keep the default stream if pinning to the built-in fails */ }
    }
  }
  // Force 16 kHz so Rust receives the pipeline's native rate (no resampler).
  audioCtx = new AudioContext({ sampleRate: 16000 });
  // Mobile WebViews start the context 'suspended' when it's created after an
  // await has consumed the tap's transient activation — resume() or the worklet
  // never runs (silent: no PCM, no error). The readout shows the real state.
  await audioCtx.resume();
  report();
  if (audioCtx.sampleRate !== 16000) console.warn('[mic] AudioContext rate', audioCtx.sampleRate, '!= 16000; Rust will resample');
  const url = URL.createObjectURL(new Blob([WORKLET_SRC], { type: 'application/javascript' }));
  await audioCtx.audioWorklet.addModule(url);
  URL.revokeObjectURL(url);
  const srcNode = audioCtx.createMediaStreamSource(stream);
  // User-adjustable input gain (the sensitivity knob), live via setMicGain().
  // f32_to_i16 on the Rust side clamps, so gain > 1 hard-clips rather than wraps.
  gainNode = audioCtx.createGain();
  gainNode.gain.value = gain;
  node = new AudioWorkletNode(audioCtx, 'mic-capture');
  // Each ~100 ms batch → Rust Vec<f32> plus the actual WebView AudioContext rate.
  node.port.onmessage = (e) => {
    batches++;
    report();
    const payload = e.data as { samples: number[]; sampleRate?: number } | number[];
    const samples = Array.isArray(payload) ? payload : payload.samples;
    const sampleRate = Array.isArray(payload) ? audioCtx?.sampleRate : payload.sampleRate;
    void invoke('feed_mic_pcm', { samples, sample_rate: sampleRate });
  };
  srcNode.connect(gainNode);
  gainNode.connect(node);
  node.connect(audioCtx.destination); // keep the graph pulling; the worklet outputs silence (no echo)
}

/** Live-adjust the input gain (1.0 = unity). No-op when not capturing. */
export function setMicGain(gain: number): void {
  if (gainNode) gainNode.gain.value = gain;
}

/** Live-toggle automatic gain control on the active mic track. Best-effort: some
 *  WebViews ignore live applyConstraints, in which case it takes effect on the
 *  next startMicCapture from the persisted config. No-op when not capturing. */
export function setMicAgc(agc: boolean): void {
  void stream?.getAudioTracks()[0]?.applyConstraints({ autoGainControl: agc }).catch(() => {});
}

export function stopMicCapture(): void {
  node?.disconnect();
  gainNode?.disconnect();
  stream?.getTracks().forEach((t) => t.stop());
  void audioCtx?.close();
  node = null;
  gainNode = null;
  stream = null;
  audioCtx = null;
}

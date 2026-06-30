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
  stream = await navigator.mediaDevices.getUserMedia({
    audio: { autoGainControl: agc, noiseSuppression: false, echoCancellation: false },
  });
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

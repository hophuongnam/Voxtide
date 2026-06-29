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
      if (this.buf.length >= 1600) { this.port.postMessage(this.buf); this.buf = []; }
    }
    return true;
  }
}
registerProcessor('mic-capture', MicCapture);
`;

let audioCtx: AudioContext | null = null;
let node: AudioWorkletNode | null = null;
let stream: MediaStream | null = null;

/** Start capturing the mic and streaming PCM to Rust. Throws if mic permission
 *  is denied (NotAllowedError) — the caller surfaces that. */
export async function startMicCapture(): Promise<void> {
  stream = await navigator.mediaDevices.getUserMedia({ audio: true });
  // Force 16 kHz so Rust receives the pipeline's native rate (no resampler).
  audioCtx = new AudioContext({ sampleRate: 16000 });
  if (audioCtx.sampleRate !== 16000) {
    console.warn('[mic] AudioContext rate', audioCtx.sampleRate, '!= 16000; audio may be pitch-shifted');
  }
  const url = URL.createObjectURL(new Blob([WORKLET_SRC], { type: 'application/javascript' }));
  await audioCtx.audioWorklet.addModule(url);
  URL.revokeObjectURL(url);
  const srcNode = audioCtx.createMediaStreamSource(stream);
  node = new AudioWorkletNode(audioCtx, 'mic-capture');
  // Each ~100 ms batch (plain number[]) → Rust Vec<f32>. JSON-serializable as-is.
  node.port.onmessage = (e) => { void invoke('feed_mic_pcm', { samples: e.data }); };
  srcNode.connect(node);
  node.connect(audioCtx.destination); // keep the graph pulling; the worklet outputs silence (no echo)
}

export function stopMicCapture(): void {
  node?.disconnect();
  stream?.getTracks().forEach((t) => t.stop());
  void audioCtx?.close();
  node = null;
  stream = null;
  audioCtx = null;
}

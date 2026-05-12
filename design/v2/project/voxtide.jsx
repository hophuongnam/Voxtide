// Voxtide UI v2 — themed (dark + light), grouped sidebar, hover-reveal overlay,
// speaker chips in both modes, variable-width status bar, icon system.

// ── theme tokens via CSS variables ──────────────────────────────────────────
(function injectVoxtideTheme() {
  if (typeof document === 'undefined' || document.getElementById('vt-theme')) return;
  const s = document.createElement('style');
  s.id = 'vt-theme';
  s.textContent = `
    .vt-theme-dark {
      --vt-bg: oklch(0.16 0.005 250);
      --vt-bg-deep: oklch(0.12 0.005 250);
      --vt-surface: oklch(0.20 0.006 250);
      --vt-surface2: oklch(0.24 0.007 250);
      --vt-surface3: oklch(0.28 0.008 250);
      --vt-border: oklch(0.30 0.008 250);
      --vt-border-hi: oklch(0.40 0.010 250);
      --vt-line-sep: oklch(0.30 0.008 250 / 0.5);
      --vt-text: oklch(0.97 0.003 250);
      --vt-muted: oklch(0.70 0.005 250);
      --vt-subtle: oklch(0.52 0.005 250);
      --vt-dim: oklch(0.38 0.005 250);
      --vt-accent: oklch(0.80 0.13 205);
      --vt-accent-dim: oklch(0.42 0.08 205);
      --vt-accent-ink: oklch(0.18 0.04 205);
      --vt-accent-tint-10: oklch(0.80 0.13 205 / 0.10);
      --vt-accent-tint-25: oklch(0.80 0.13 205 / 0.25);
      --vt-accent-tint-50: oklch(0.80 0.13 205 / 0.50);
      --vt-rec: oklch(0.72 0.20 25);
      --vt-rec-glow: oklch(0.72 0.20 25 / 0.20);
      --vt-warn: oklch(0.82 0.14 80);
      --vt-warn-tint: oklch(0.82 0.14 80 / 0.10);
      --vt-warn-border: oklch(0.82 0.14 80 / 0.4);
      --vt-ok: oklch(0.78 0.13 155);
      --vt-overlay-bg: oklch(0.13 0.005 250 / 0.88);
      --vt-overlay-border: oklch(0.50 0.010 250 / 0.5);
      --vt-overlay-shadow: 0 1px 0 rgba(255,255,255,0.06) inset, 0 20px 50px -10px rgba(0,0,0,0.5), 0 8px 20px -8px rgba(0,0,0,0.6);
      --vt-window-shadow: 0 0 0 0.5px rgba(0,0,0,0.6), 0 1px 0 rgba(255,255,255,0.04) inset, 0 30px 60px -20px rgba(0,0,0,0.6), 0 18px 36px -18px rgba(0,0,0,0.5);
      --vt-traffic-dim: oklch(0.30 0.005 250);
      --vt-speaker-a: oklch(0.80 0.13 205);
      --vt-speaker-b: oklch(0.78 0.13 25);
      --vt-speaker-c: oklch(0.78 0.13 155);
      --vt-speaker-d: oklch(0.82 0.14 80);
      --vt-speaker-ink: oklch(0.14 0.02 250);
      --vt-btn-inner-hi: rgba(255,255,255,0.15);
    }
    .vt-theme-light {
      --vt-bg: oklch(0.985 0.003 250);
      --vt-bg-deep: oklch(0.965 0.004 250);
      --vt-surface: oklch(0.95 0.005 250);
      --vt-surface2: oklch(0.92 0.006 250);
      --vt-surface3: oklch(0.88 0.007 250);
      --vt-border: oklch(0.85 0.008 250);
      --vt-border-hi: oklch(0.72 0.010 250);
      --vt-line-sep: oklch(0.85 0.008 250 / 0.7);
      --vt-text: oklch(0.20 0.005 250);
      --vt-muted: oklch(0.42 0.005 250);
      --vt-subtle: oklch(0.55 0.005 250);
      --vt-dim: oklch(0.70 0.005 250);
      --vt-accent: oklch(0.55 0.16 205);
      --vt-accent-dim: oklch(0.42 0.10 205);
      --vt-accent-ink: oklch(0.99 0.005 250);
      --vt-accent-tint-10: oklch(0.55 0.16 205 / 0.10);
      --vt-accent-tint-25: oklch(0.55 0.16 205 / 0.25);
      --vt-accent-tint-50: oklch(0.55 0.16 205 / 0.50);
      --vt-rec: oklch(0.55 0.20 25);
      --vt-rec-glow: oklch(0.55 0.20 25 / 0.18);
      --vt-warn: oklch(0.65 0.16 80);
      --vt-warn-tint: oklch(0.65 0.16 80 / 0.12);
      --vt-warn-border: oklch(0.65 0.16 80 / 0.45);
      --vt-ok: oklch(0.55 0.14 155);
      --vt-overlay-bg: oklch(0.985 0.005 250 / 0.92);
      --vt-overlay-border: oklch(0.65 0.010 250 / 0.4);
      --vt-overlay-shadow: 0 1px 0 rgba(255,255,255,0.6) inset, 0 20px 50px -10px rgba(40,55,80,0.18), 0 8px 20px -8px rgba(40,55,80,0.20);
      --vt-window-shadow: 0 0 0 0.5px rgba(40,55,80,0.18), 0 1px 0 rgba(255,255,255,0.7) inset, 0 30px 60px -20px rgba(40,55,80,0.18), 0 18px 36px -18px rgba(40,55,80,0.15);
      --vt-traffic-dim: oklch(0.78 0.005 250);
      --vt-speaker-a: oklch(0.55 0.16 205);
      --vt-speaker-b: oklch(0.55 0.20 25);
      --vt-speaker-c: oklch(0.55 0.14 155);
      --vt-speaker-d: oklch(0.65 0.16 80);
      --vt-speaker-ink: oklch(0.99 0.005 250);
      --vt-btn-inner-hi: rgba(255,255,255,0.6);
    }
  `;
  document.head.appendChild(s);
})();

const V = {
  font: '"Geist", -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
  mono: '"Geist Mono", "JetBrains Mono", ui-monospace, monospace',
  bg:        'var(--vt-bg)',
  bgDeep:    'var(--vt-bg-deep)',
  surface:   'var(--vt-surface)',
  surface2:  'var(--vt-surface2)',
  surface3:  'var(--vt-surface3)',
  border:    'var(--vt-border)',
  borderHi:  'var(--vt-border-hi)',
  line:      'var(--vt-line-sep)',
  text:      'var(--vt-text)',
  muted:     'var(--vt-muted)',
  subtle:    'var(--vt-subtle)',
  dim:       'var(--vt-dim)',
  accent:    'var(--vt-accent)',
  accentDim: 'var(--vt-accent-dim)',
  accentInk: 'var(--vt-accent-ink)',
  accent10:  'var(--vt-accent-tint-10)',
  accent25:  'var(--vt-accent-tint-25)',
  accent50:  'var(--vt-accent-tint-50)',
  rec:       'var(--vt-rec)',
  recGlow:   'var(--vt-rec-glow)',
  warn:      'var(--vt-warn)',
  warnTint:  'var(--vt-warn-tint)',
  warnBd:    'var(--vt-warn-border)',
  ok:        'var(--vt-ok)',
  ovBg:      'var(--vt-overlay-bg)',
  ovBd:      'var(--vt-overlay-border)',
  ovShadow:  'var(--vt-overlay-shadow)',
  winShadow: 'var(--vt-window-shadow)',
  trafficDim:'var(--vt-traffic-dim)',
  speakerInk:'var(--vt-speaker-ink)',
};

const SPEAKER_COLORS = ['var(--vt-speaker-a)', 'var(--vt-speaker-b)', 'var(--vt-speaker-c)', 'var(--vt-speaker-d)'];
const speakerColor = (s) => SPEAKER_COLORS[(s.charCodeAt(0) - 65) % 4];

// ─── primitives ─────────────────────────────────────────────────────────────

const TrafficLights = () => (
  <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
    {['#ff5f57', '#febc2e', '#28c840'].map(c => (
      <div key={c} style={{
        width: 12, height: 12, borderRadius: '50%', background: c,
        boxShadow: 'inset 0 0 0 0.5px rgba(0,0,0,0.18)',
      }} />
    ))}
  </div>
);

const Icon = ({ d, size = 14, stroke = 1.6, color = 'currentColor' }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none"
    stroke={color} strokeWidth={stroke} strokeLinecap="round" strokeLinejoin="round">
    <path d={d} />
  </svg>
);
const Icons = {
  mic:      "M12 3a3 3 0 0 0-3 3v6a3 3 0 0 0 6 0V6a3 3 0 0 0-3-3z M5 11a7 7 0 0 0 14 0 M12 18v3 M8 21h8",
  speaker:  "M3 10v4h4l5 4V6l-5 4H3z M16 8a5 5 0 0 1 0 8 M19.5 5a9 9 0 0 1 0 14",
  overlay:  "M3 5h12v10H3z M9 9h12v10H9z",
  cog:      "M12 8.5a3.5 3.5 0 1 0 0 7 3.5 3.5 0 0 0 0-7z M19.4 13a7.5 7.5 0 0 0 0-2l2-1.5-2-3.4-2.4.8a7.5 7.5 0 0 0-1.7-1L14.8 3h-5.6l-.5 2.6a7.5 7.5 0 0 0-1.7 1l-2.4-.8-2 3.4 2 1.5a7.5 7.5 0 0 0 0 2l-2 1.5 2 3.4 2.4-.8a7.5 7.5 0 0 0 1.7 1l.5 2.6h5.6l.5-2.6a7.5 7.5 0 0 0 1.7-1l2.4.8 2-3.4-2-1.5z",
  search:   "M10.5 4a6.5 6.5 0 1 1 0 13 6.5 6.5 0 0 1 0-13z M20 20l-4.5-4.5",
  play:     "M6 4l14 8-14 8V4z",
  swap:     "M7 7h13l-3-3 M17 17H4l3 3",
  chevron:  "M6 9l6 6 6-6",
  close:    "M6 6l12 12 M18 6L6 18",
  plus:     "M12 5v14 M5 12h14",
  arrow:    "M5 12h14 M14 7l5 5-5 5",
  key:      "M14 8a4 4 0 1 1-4 4 M14 8l6 0 m0 0v3 m-3-3v3 M10 12L3 19v2h2l1-1h2v-2h2l1-1",
};

// ─── window chrome ──────────────────────────────────────────────────────────

function VoxWindow({ width = 920, height = 600, children }) {
  return (
    <div style={{
      width, height, borderRadius: 12, overflow: 'hidden',
      background: V.bg, color: V.text, fontFamily: V.font,
      boxShadow: V.winShadow,
      display: 'flex', flexDirection: 'column',
      border: `0.5px solid ${V.border}`,
    }}>{children}</div>
  );
}

// ─── wordmark glyph (reused as app icon at small sizes) ─────────────────────
function WaveGlyph({ size = 12, color, bars = 5 }) {
  // Generates a centered sound-wave: heights vary symmetrically.
  const heights5 = [0.4, 0.7, 1.0, 0.7, 0.4];
  const heights3 = [0.55, 1.0, 0.55];
  const hs = bars === 3 ? heights3 : heights5;
  const w = size, h = size;
  const barW = Math.max(1, Math.round(size / (hs.length * 2.5)));
  const gap = Math.max(1, Math.round(size / (hs.length * 4)));
  const totalW = hs.length * barW + (hs.length - 1) * gap;
  const startX = (w - totalW) / 2;
  return (
    <svg width={size} height={size} viewBox={`0 0 ${w} ${h}`}>
      {hs.map((v, i) => {
        const bh = v * h * 0.7;
        return (
          <rect key={i}
            x={startX + i * (barW + gap)}
            y={(h - bh) / 2}
            width={barW} height={bh}
            rx={Math.max(0.5, barW * 0.3)}
            fill={color} />
        );
      })}
    </svg>
  );
}

// ─── top toolbar ────────────────────────────────────────────────────────────

function ModeToggle({ mode, onChange }) {
  return (
    <div style={{
      display: 'inline-flex', padding: 2, background: V.surface,
      borderRadius: 8, border: `0.5px solid ${V.border}`,
    }}>
      {[
        { id: 'meeting', label: 'Meeting' },
        { id: 'conversation', label: 'Conversation' },
      ].map(m => {
        const active = mode === m.id;
        return (
          <button key={m.id} onClick={() => onChange(m.id)} style={{
            padding: '5px 12px', borderRadius: 6, border: 'none', cursor: 'pointer',
            background: active ? V.surface3 : 'transparent',
            color: active ? V.text : V.muted,
            boxShadow: active ? `0 0 0 0.5px ${V.borderHi}, 0 1px 0 ${V.btnInnerHi || 'transparent'} inset` : 'none',
            fontFamily: V.font, fontSize: 12, fontWeight: 500,
          }}>{m.label}</button>
        );
      })}
    </div>
  );
}

function LangChip({ code, name, mine }) {
  return (
    <button style={{
      display: 'inline-flex', alignItems: 'center', gap: 8,
      padding: '5px 10px 5px 8px', borderRadius: 7,
      background: V.surface, border: `0.5px solid ${V.border}`,
      color: V.text, fontFamily: V.font, fontSize: 12, cursor: 'pointer',
      position: 'relative',
    }}>
      <span style={{
        fontFamily: V.mono, fontSize: 10, fontWeight: 600,
        background: V.surface3, color: V.muted,
        padding: '2px 5px', borderRadius: 4, letterSpacing: 0.4,
      }}>{code}</span>
      <span style={{ color: V.text }}>{name}</span>
      {mine && <span style={{
        position: 'absolute', top: -5, right: -5,
        background: V.accent, color: V.accentInk,
        fontSize: 8.5, fontWeight: 700, padding: '1px 4px',
        borderRadius: 3, letterSpacing: 0.3,
      }}>YOU</span>}
      <Icon d={Icons.chevron} size={11} color={V.muted} />
    </button>
  );
}

function LangPair({ a, b, mine = 'a' }) {
  return (
    <div style={{ display: 'inline-flex', alignItems: 'center', gap: 6 }}>
      <LangChip {...a} mine={mine === 'a'} />
      <button style={{
        width: 22, height: 22, borderRadius: 6, background: 'transparent',
        border: 'none', color: V.subtle, cursor: 'pointer',
        display: 'inline-flex', alignItems: 'center', justifyContent: 'center',
      }}>
        <Icon d={Icons.swap} size={14} stroke={1.4} />
      </button>
      <LangChip {...b} mine={mine === 'b'} />
    </div>
  );
}

function AudioSource({ mode }) {
  const meeting = { icon: Icons.speaker, name: 'Zoom Meeting', sub: 'System audio' };
  const conv = { icon: Icons.mic, name: 'MacBook Pro Mic', sub: 'Built-in' };
  const src = mode === 'meeting' ? meeting : conv;
  return (
    <button style={{
      display: 'inline-flex', alignItems: 'center', gap: 8,
      padding: '5px 10px', borderRadius: 7,
      background: V.surface, border: `0.5px solid ${V.border}`,
      color: V.text, fontFamily: V.font, fontSize: 12, cursor: 'pointer',
      maxWidth: 200,
    }}>
      <Icon d={src.icon} size={13} color={V.muted} />
      <span style={{
        whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis',
      }}>{src.name}</span>
      <span style={{ color: V.subtle, fontSize: 11 }}>·</span>
      <span style={{ color: V.subtle, fontSize: 11 }}>{src.sub}</span>
      <Icon d={Icons.chevron} size={11} color={V.muted} />
    </button>
  );
}

function IconBtn({ d, active, onClick, title }) {
  return (
    <button onClick={onClick} title={title} style={{
      width: 30, height: 30, borderRadius: 7,
      border: `0.5px solid ${active ? V.borderHi : V.border}`,
      background: active ? V.surface3 : V.surface,
      color: active ? V.text : V.muted, cursor: 'pointer',
      display: 'inline-flex', alignItems: 'center', justifyContent: 'center',
    }}><Icon d={d} size={14} /></button>
  );
}

function PrimaryBtn({ children, recording, onClick }) {
  return (
    <button onClick={onClick} style={{
      display: 'inline-flex', alignItems: 'center', gap: 7,
      padding: '6px 12px 6px 10px', borderRadius: 7,
      background: recording ? V.rec : V.accent,
      color: recording ? '#fff' : V.accentInk,
      border: 'none', cursor: 'pointer',
      fontFamily: V.font, fontSize: 12, fontWeight: 600, letterSpacing: -0.1,
      boxShadow: '0 1px 0 var(--vt-btn-inner-hi) inset, 0 6px 14px -6px rgba(0,0,0,0.35)',
    }}>
      {recording
        ? <div style={{ width: 8, height: 8, borderRadius: 2, background: '#fff' }} />
        : <Icon d={Icons.play} size={11} />}
      {children}
    </button>
  );
}

function Toolbar({ mode, setMode, recording, setRecording }) {
  const langs = {
    meeting: { a: { code: 'EN', name: 'English' }, b: { code: 'VI', name: 'Vietnamese' }, mine: 'b' },
    conversation: { a: { code: 'EN', name: 'English' }, b: { code: 'JA', name: 'Japanese' }, mine: 'a' },
  }[mode];
  return (
    <div style={{
      height: 48, display: 'flex', alignItems: 'center', gap: 10,
      padding: '0 12px 0 14px', borderBottom: `0.5px solid ${V.border}`,
      background: V.bg,
    }}>
      <TrafficLights />
      <div style={{ width: 1, height: 18, background: V.border, margin: '0 4px' }} />
      <div style={{ display: 'flex', alignItems: 'center', gap: 7 }}>
        <div style={{
          width: 18, height: 18, borderRadius: 5,
          background: `linear-gradient(135deg, ${V.accent}, ${V.accentDim})`,
          display: 'flex', alignItems: 'center', justifyContent: 'center',
        }}>
          <WaveGlyph size={11} color={V.accentInk} bars={5} />
        </div>
        <span style={{
          fontSize: 13, fontWeight: 600, letterSpacing: -0.2, color: V.text,
        }}>Voxtide</span>
      </div>
      <div style={{ width: 1, height: 18, background: V.border, margin: '0 4px 0 2px' }} />
      <ModeToggle mode={mode} onChange={setMode} />
      <div style={{ marginLeft: 8 }}><LangPair {...langs} /></div>
      <div style={{ marginLeft: 6 }}><AudioSource mode={mode} /></div>
      <div style={{ flex: 1 }} />
      <IconBtn d={Icons.overlay} active title="Show overlay" />
      <IconBtn d={Icons.cog} title="Settings" />
      <PrimaryBtn recording={recording} onClick={() => setRecording(!recording)}>
        {recording ? 'Stop' : 'Start'}
      </PrimaryBtn>
    </div>
  );
}

// ─── sidebar (date-grouped) ─────────────────────────────────────────────────

const SESSIONS = [
  { group: 'Today', time: '14:22', dur: '38m', a: 'EN', b: 'VI', mode: 'meeting', active: true,
    preview: 'So the next milestone is to ship the prototype by end of week…' },
  { group: 'Today', time: '11:48', dur: '4m', a: 'EN', b: 'ES', mode: 'conversation',
    preview: '¿Me puede ayudar con esto? — Could you help me with this?' },
  { group: 'Today', time: '09:05', dur: '12m', a: 'EN', b: 'JA', mode: 'conversation',
    preview: 'すみません、駅はどこですか — Excuse me, where is the station?' },
  { group: 'Yesterday', time: '17:30', dur: '1h 04m', a: 'EN', b: 'VI', mode: 'meeting',
    preview: 'Quarterly review with the Hanoi team. Q3 targets revisited.' },
  { group: 'Yesterday', time: '10:12', dur: '8m', a: 'KO', b: 'EN', mode: 'conversation',
    preview: '커피 한 잔 주세요 — One coffee, please.' },
  { group: 'This week', time: 'May 10 · 16:04', dur: '22m', a: 'KO', b: 'EN', mode: 'meeting',
    preview: '안녕하세요 — Hi everyone, thanks for joining today\u2019s sync.' },
  { group: 'This week', time: 'May 9 · 14:00', dur: '7m', a: 'EN', b: 'ES', mode: 'conversation',
    preview: '¿Dónde está el baño? — Where is the restroom?' },
  { group: 'This week', time: 'May 8 · 09:30', dur: '46m', a: 'EN', b: 'VI', mode: 'meeting',
    preview: 'Architecture review — Tauri vs Electron tradeoffs, audio routing.' },
  { group: 'Earlier', time: 'May 4 · 11:20', dur: '15m', a: 'EN', b: 'DE', mode: 'meeting',
    preview: 'Guten Morgen — Stand-up with the Berlin team.' },
  { group: 'Earlier', time: 'May 2 · 19:45', dur: '53m', a: 'EN', b: 'FR', mode: 'meeting',
    preview: 'Bonjour à tous — design review of the overlay window.' },
  { group: 'Earlier', time: 'Apr 28 · 08:15', dur: '6m', a: 'EN', b: 'JA', mode: 'conversation',
    preview: 'コーヒーをお願いします — A coffee, please.' },
  { group: 'Earlier', time: 'Apr 22 · 13:50', dur: '1h 22m', a: 'ZH', b: 'EN', mode: 'meeting',
    preview: '你好 — Customer onboarding session with Shanghai.' },
];

const LangTag = () => ({
  fontFamily: V.mono, fontSize: 9.5, fontWeight: 600, color: V.muted,
  background: V.surface, padding: '1px 4px', borderRadius: 3,
  letterSpacing: 0.4, border: `0.5px solid ${V.border}`,
});

function Sidebar({ activeIdx = 0 }) {
  // group by date
  const grouped = [];
  let curr = null;
  SESSIONS.forEach((s, i) => {
    if (!curr || curr.group !== s.group) {
      curr = { group: s.group, items: [] };
      grouped.push(curr);
    }
    curr.items.push({ ...s, _i: i });
  });

  return (
    <div style={{
      width: 240, flexShrink: 0, borderRight: `0.5px solid ${V.border}`,
      background: V.bgDeep, display: 'flex', flexDirection: 'column',
    }}>
      <div style={{ padding: '12px 12px 8px' }}>
        <div style={{
          display: 'flex', alignItems: 'center', gap: 7,
          padding: '6px 9px', borderRadius: 7,
          background: V.surface, border: `0.5px solid ${V.border}`,
        }}>
          <Icon d={Icons.search} size={13} color={V.subtle} />
          <input placeholder="Search transcripts…" style={{
            flex: 1, background: 'transparent', border: 'none', outline: 'none',
            color: V.text, fontFamily: V.font, fontSize: 12, minWidth: 0,
          }} />
          <span style={{
            fontFamily: V.mono, fontSize: 10, color: V.subtle,
            padding: '1px 4px', borderRadius: 3, border: `0.5px solid ${V.border}`,
          }}>⌘K</span>
        </div>
      </div>
      <div style={{
        padding: '4px 14px 6px', display: 'flex', justifyContent: 'space-between',
        alignItems: 'center',
      }}>
        <span style={{
          fontSize: 10, fontWeight: 600, letterSpacing: 0.6,
          color: V.subtle, textTransform: 'uppercase',
        }}>History</span>
        <button style={{
          width: 18, height: 18, borderRadius: 4, background: 'transparent',
          border: 'none', color: V.subtle, cursor: 'pointer',
          display: 'inline-flex', alignItems: 'center', justifyContent: 'center',
        }}><Icon d={Icons.plus} size={12} /></button>
      </div>
      <div style={{ flex: 1, overflow: 'auto', padding: '0 8px 8px', position: 'relative' }}>
        {grouped.map((g, gi) => (
          <div key={gi}>
            <div style={{
              position: 'sticky', top: 0, zIndex: 2,
              padding: '6px 6px 4px',
              background: `linear-gradient(${V.bgDeep} 70%, transparent 100%)`,
              fontFamily: V.mono, fontSize: 9, fontWeight: 600,
              color: V.subtle, letterSpacing: 0.6, textTransform: 'uppercase',
            }}>{g.group}</div>
            {g.items.map(s => {
              const active = s._i === activeIdx;
              return (
                <div key={s._i} style={{
                  padding: '8px 9px', borderRadius: 7, marginBottom: 2,
                  background: active ? V.surface2 : 'transparent',
                  border: `0.5px solid ${active ? V.border : 'transparent'}`,
                  cursor: 'pointer',
                }}>
                  <div style={{
                    display: 'flex', alignItems: 'center', justifyContent: 'space-between',
                    marginBottom: 4,
                  }}>
                    <span style={{ fontSize: 11, color: V.muted }}>{s.time}</span>
                    <span style={{ fontFamily: V.mono, fontSize: 10, color: V.subtle }}>{s.dur}</span>
                  </div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 5, marginBottom: 5 }}>
                    <span style={LangTag()}>{s.a}</span>
                    <Icon d={Icons.arrow} size={10} color={V.subtle} />
                    <span style={LangTag()}>{s.b}</span>
                    <span style={{
                      marginLeft: 'auto', fontSize: 9.5, color: V.subtle,
                      textTransform: 'uppercase', letterSpacing: 0.5,
                    }}>{s.mode}</span>
                    {s.active && <div style={{
                      width: 6, height: 6, borderRadius: '50%', background: V.rec,
                      boxShadow: `0 0 0 2px ${V.recGlow}`,
                    }} />}
                  </div>
                  <div style={{
                    fontSize: 11, color: V.muted, lineHeight: 1.4,
                    display: '-webkit-box', WebkitLineClamp: 2, WebkitBoxOrient: 'vertical',
                    overflow: 'hidden', textOverflow: 'ellipsis',
                  }}>{s.preview}</div>
                </div>
              );
            })}
          </div>
        ))}
      </div>
    </div>
  );
}

// ─── transcript content ─────────────────────────────────────────────────────

// Meeting: speaker chips on every line — multi-participant call
const MEETING_TRANSCRIPT = [
  { sp: 'A', name: 'Linh', o: "Alright, so let's start with the engineering update.",
    t: "Được rồi, hãy bắt đầu với cập nhật từ đội kỹ thuật.", t_ms: '14:22:08' },
  { sp: 'B', name: 'Marcus', o: "We're tracking ahead on the audio capture layer — both macOS taps and WASAPI loopback are stable.",
    t: "Chúng tôi đang đi trước tiến độ ở lớp thu âm thanh — cả Core Audio taps và WASAPI loopback đều ổn định.", t_ms: '14:22:14' },
  { sp: 'B', name: 'Marcus', o: "Latency end-to-end is hovering around two hundred sixty milliseconds.",
    t: "Độ trễ tổng từ đầu đến cuối hiện đang khoảng hai trăm sáu mươi mili-giây.", t_ms: '14:22:23' },
  { sp: 'A', name: 'Linh', o: "That's well under our sub-second target, so we're comfortable there.",
    t: "Con số đó thấp hơn nhiều so với mục tiêu dưới một giây, nên chúng ta đang ở vị thế tốt.", t_ms: '14:22:31' },
  { sp: 'C', name: 'Aisha', o: "Next milestone is to ship the prototype by end of week and start dogfooding on Monday.",
    t: "Cột mốc tiếp theo là phát hành bản nguyên mẫu vào cuối tuần và bắt đầu dùng nội bộ từ thứ Hai.", t_ms: '14:22:40', live: true },
];

const CONV_TRANSCRIPT = [
  { sp: 'A', a: 'EN', b: 'JA',
    o: "Excuse me, do you know the way to Shibuya station?",
    t: "すみません、渋谷駅への道をご存じですか？", t_ms: '09:05:12' },
  { sp: 'B', a: 'JA', b: 'EN',
    o: "はい、この道をまっすぐ行って、二つ目の信号を右に曲がってください。",
    t: "Yes — go straight down this road, then turn right at the second traffic light.", t_ms: '09:05:18' },
  { sp: 'A', a: 'EN', b: 'JA',
    o: "Thank you — about how many minutes is it on foot?",
    t: "ありがとうございます — 歩いて何分くらいかかりますか？", t_ms: '09:05:26' },
  { sp: 'B', a: 'JA', b: 'EN',
    o: "そうですね、十分くらいだと思います。",
    t: "Let me think — about ten minutes, I'd say.", t_ms: '09:05:33', live: true },
];

function LevelMeter({ active }) {
  const [tick, setTick] = React.useState(0);
  React.useEffect(() => {
    if (!active) return;
    const id = setInterval(() => setTick(t => t + 1), 80);
    return () => clearInterval(id);
  }, [active]);
  const bars = Array.from({ length: 14 }, (_, i) => {
    if (!active) return 0.12;
    const v = 0.25 + 0.5 * Math.abs(Math.sin((tick + i * 1.3) * 0.6 + i));
    return Math.min(1, v * (0.6 + 0.4 * Math.sin(tick * 0.2 + i * 0.4)));
  });
  return (
    <div style={{ display: 'inline-flex', alignItems: 'center', gap: 2, height: 14 }}>
      {bars.map((h, i) => (
        <div key={i} style={{
          width: 2, height: `${100 * h}%`, minHeight: 2, borderRadius: 1,
          background: active ? (h > 0.7 ? V.accent : V.accentDim) : V.dim,
          transition: 'height .08s',
        }} />
      ))}
    </div>
  );
}

function StatusBar({ recording, mode, elapsed = '00:38:24', width = 920 }) {
  // hide priority right-to-left: format → latency → model → translation summary
  const showFormat = width >= 900;
  const showLatency = width >= 700;
  const showModel = width >= 580;
  const showSummary = width >= 480;
  const sep = <span style={{ color: V.dim }}>│</span>;
  return (
    <div style={{
      height: 28, borderTop: `0.5px solid ${V.border}`, background: V.bgDeep,
      display: 'flex', alignItems: 'center', gap: 12,
      padding: '0 12px',
      fontFamily: V.mono, fontSize: 10.5, color: V.subtle, letterSpacing: 0.2,
      whiteSpace: 'nowrap',
    }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
        <div style={{
          width: 7, height: 7, borderRadius: '50%',
          background: recording ? V.rec : V.dim,
          boxShadow: recording ? `0 0 0 3px ${V.recGlow}` : 'none',
        }} />
        <span style={{ color: recording ? V.text : V.subtle, textTransform: 'uppercase' }}>
          {recording ? 'REC' : 'IDLE'}
        </span>
        <span>{recording ? elapsed : '—'}</span>
      </div>
      {sep}
      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        <LevelMeter active={recording} />
        <span>{recording ? '-18 dB' : ''}</span>
      </div>
      {showModel && <>{sep}<span>SONIOX · stt-rt-v4</span></>}
      {showLatency && <>{sep}<span>{recording ? '262 ms' : 'ws idle'}</span></>}
      <div style={{ flex: 1 }} />
      {showSummary && <span>{mode === 'meeting' ? 'one_way → VI' : 'two_way · EN ⇄ JA'}</span>}
      {showFormat && <>{sep}<span>16 kHz · mono · s16le</span></>}
    </div>
  );
}

// ─── transcript columns / line ──────────────────────────────────────────────

function Column({ label, code, sub, children, accent }) {
  return (
    <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0 }}>
      <div style={{
        height: 38, padding: '0 16px', display: 'flex', alignItems: 'center', gap: 8,
        borderBottom: `0.5px solid ${V.border}`, background: V.bg,
      }}>
        <span style={{ fontSize: 11, fontWeight: 600, color: V.text, letterSpacing: -0.1 }}>{label}</span>
        <span style={{
          fontFamily: V.mono, fontSize: 9.5, fontWeight: 600, letterSpacing: 0.4,
          color: accent ? V.accent : V.muted,
          background: accent ? V.accent10 : V.surface,
          padding: '2px 5px', borderRadius: 3,
          border: `0.5px solid ${accent ? V.accent25 : V.border}`,
        }}>{code}</span>
        <span style={{ fontSize: 11, color: V.subtle }}>{sub}</span>
      </div>
      <div style={{ flex: 1, overflow: 'auto', padding: '4px 0 12px' }}>
        {children}
      </div>
    </div>
  );
}

function SpeakerChip({ sp, name, lang }) {
  return (
    <div style={{ display: 'inline-flex', alignItems: 'center', gap: 6, marginBottom: 5 }}>
      <div style={{
        width: 16, height: 16, borderRadius: 4,
        background: speakerColor(sp), color: V.speakerInk,
        fontFamily: V.font, fontSize: 9.5, fontWeight: 700,
        display: 'inline-flex', alignItems: 'center', justifyContent: 'center',
      }}>{sp}</div>
      {name && <span style={{ fontSize: 11, color: V.muted, fontWeight: 500 }}>{name}</span>}
      <span style={{
        fontFamily: V.mono, fontSize: 9.5, color: V.subtle, letterSpacing: 0.4,
      }}>{lang}</span>
    </div>
  );
}

function Line({ ts, text, translated, sp, name, lang, live }) {
  return (
    <div style={{
      padding: '10px 16px',
      display: 'grid', gridTemplateColumns: '52px 1fr', gap: 10,
      borderTop: `0.5px solid ${V.line}`,
    }}>
      <div style={{
        fontFamily: V.mono, fontSize: 10, color: V.dim, letterSpacing: 0.3, paddingTop: 3,
      }}>{ts}</div>
      <div>
        {sp && <SpeakerChip sp={sp} name={name} lang={lang} />}
        <div style={{
          fontSize: 13.5, lineHeight: 1.55,
          color: live ? V.muted : V.text,
        }}>
          {text}
          {live && <span style={{
            display: 'inline-block', width: 8, height: 14, marginLeft: 2,
            background: translated ? V.accent : V.muted, verticalAlign: -2,
            animation: 'vt-blink 0.9s steps(2) infinite',
          }} />}
        </div>
      </div>
    </div>
  );
}

function MeetingTranscript({ recording }) {
  const final = MEETING_TRANSCRIPT.slice(0, -1);
  const liveLine = MEETING_TRANSCRIPT[MEETING_TRANSCRIPT.length - 1];
  const [n, setN] = React.useState(0);
  React.useEffect(() => {
    if (!recording) return;
    const id = setInterval(() => setN(x => (x + 1) % 130), 90);
    return () => clearInterval(id);
  }, [recording]);
  const oLive = recording ? liveLine.o.slice(0, Math.floor(n * 0.7)) : '';
  const tLive = recording ? liveLine.t.slice(0, Math.max(0, Math.floor((n - 25) * 0.7))) : '';

  return (
    <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
      <Column label="Original" code="EN · multi-speaker" sub="auto-detected · diarized">
        {final.map((l, i) => (
          <Line key={i} ts={l.t_ms} text={l.o} sp={l.sp} name={l.name} lang="EN" />
        ))}
        {oLive && <Line ts="now" text={oLive} sp={liveLine.sp} name={liveLine.name} lang="EN" live />}
      </Column>
      <div style={{ width: 0.5, background: V.border }} />
      <Column label="Translation" code="VI" sub="Vietnamese — target" accent>
        {final.map((l, i) => (
          <Line key={i} ts={l.t_ms} text={l.t} translated sp={l.sp} name={l.name} lang="VI" />
        ))}
        {tLive && <Line ts="now" text={tLive} translated sp={liveLine.sp} name={liveLine.name} lang="VI" live />}
      </Column>
    </div>
  );
}

function ConversationTranscript({ recording }) {
  return (
    <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
      <Column label="Original" code="EN/JA" sub="auto-detected per turn">
        {CONV_TRANSCRIPT.map((l, i) => (
          <Line key={i} ts={l.t_ms} text={l.o} sp={l.sp} lang={l.a} live={l.live && recording} />
        ))}
      </Column>
      <div style={{ width: 0.5, background: V.border }} />
      <Column label="Translation" code="EN ⇄ JA" sub="two-way" accent>
        {CONV_TRANSCRIPT.map((l, i) => (
          <Line key={i} ts={l.t_ms} text={l.t} translated sp={l.sp} lang={l.b} live={l.live && recording} />
        ))}
      </Column>
    </div>
  );
}

// ─── empty / onboarding ─────────────────────────────────────────────────────

const kbd = () => ({
  fontFamily: V.mono, fontSize: 10.5, padding: '2px 6px',
  borderRadius: 4, background: V.surface, border: `0.5px solid ${V.border}`,
  color: V.muted,
});

function EmptyTranscript({ mode }) {
  return (
    <div style={{
      flex: 1, display: 'flex', flexDirection: 'column',
      alignItems: 'center', justifyContent: 'center', padding: 24, gap: 14,
    }}>
      <div style={{
        width: 64, height: 64, borderRadius: 14,
        background: V.surface, border: `0.5px solid ${V.border}`,
        display: 'flex', alignItems: 'center', justifyContent: 'center',
        position: 'relative',
      }}>
        <Icon d={mode === 'meeting' ? Icons.speaker : Icons.mic} size={26}
              color={V.muted} stroke={1.4} />
        <div style={{
          position: 'absolute', inset: -6, borderRadius: 18,
          border: `0.5px dashed ${V.border}`,
        }} />
      </div>
      <div style={{ fontSize: 15, fontWeight: 600, color: V.text, letterSpacing: -0.2 }}>
        {mode === 'meeting' ? 'Ready to translate a meeting' : 'Ready for a conversation'}
      </div>
      <div style={{
        fontSize: 12.5, color: V.muted, maxWidth: 380, textAlign: 'center', lineHeight: 1.5,
      }}>
        {mode === 'meeting'
          ? 'Voxtide will capture system audio from the selected source and translate the remote speaker into your language.'
          : 'Voxtide will capture your microphone and translate two-way between the chosen language pair.'}
      </div>
      <div style={{ display: 'flex', gap: 8, marginTop: 6, fontSize: 11.5, color: V.subtle }}>
        <kbd style={kbd()}>⌘</kbd><kbd style={kbd()}>⇧</kbd><kbd style={kbd()}>V</kbd>
        <span style={{ alignSelf: 'center', marginLeft: 4 }}>to start anywhere</span>
      </div>
    </div>
  );
}

function NoApiKey() {
  return (
    <div style={{
      flex: 1, display: 'flex', flexDirection: 'column',
      alignItems: 'center', justifyContent: 'center', padding: 24, gap: 12,
    }}>
      <div style={{
        width: 56, height: 56, borderRadius: 12,
        background: V.warnTint, border: `0.5px solid ${V.warnBd}`,
        color: V.warn, display: 'flex', alignItems: 'center', justifyContent: 'center',
      }}>
        <Icon d={Icons.key} size={22} stroke={1.5} />
      </div>
      <div style={{ fontSize: 15, fontWeight: 600, color: V.text }}>
        Add your Soniox API key
      </div>
      <div style={{
        fontSize: 12.5, color: V.muted, maxWidth: 380, textAlign: 'center', lineHeight: 1.5,
      }}>
        Voxtide uses your own Soniox key for real-time translation. The key is
        stored in your OS keychain and never leaves this device.
      </div>
      <div style={{ display: 'flex', gap: 8, marginTop: 8 }}>
        <button style={{
          padding: '8px 14px', borderRadius: 7, border: 'none', cursor: 'pointer',
          background: V.accent, color: V.accentInk,
          fontFamily: V.font, fontSize: 12, fontWeight: 600,
        }}>Add API key</button>
        <button style={{
          padding: '8px 14px', borderRadius: 7, cursor: 'pointer',
          background: 'transparent', color: V.muted,
          border: `0.5px solid ${V.border}`,
          fontFamily: V.font, fontSize: 12, fontWeight: 500,
        }}>Get a Soniox key →</button>
      </div>
    </div>
  );
}

// ─── main window ────────────────────────────────────────────────────────────

function MainWindow({ initialMode = 'meeting', initialRecording = true, empty = false, onboarding = false }) {
  const [mode, setMode] = React.useState(initialMode);
  const [recording, setRecording] = React.useState(initialRecording);
  return (
    <VoxWindow width={920} height={600}>
      <Toolbar mode={mode} setMode={setMode}
               recording={recording} setRecording={setRecording} />
      <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
        <Sidebar />
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0 }}>
          {onboarding
            ? <NoApiKey />
            : empty
              ? <EmptyTranscript mode={mode} />
              : mode === 'meeting'
                ? <MeetingTranscript recording={recording} />
                : <ConversationTranscript recording={recording} />}
          <StatusBar recording={recording} mode={mode} width={920 - 240} />
        </div>
      </div>
    </VoxWindow>
  );
}

// ─── overlay (hover-reveal control strip) ───────────────────────────────────

const OVERLAY_LINES = [
  'Được rồi, hãy bắt đầu với cập nhật từ đội kỹ thuật.',
  'Chúng tôi đang đi trước tiến độ ở lớp thu âm thanh — cả Core Audio taps và WASAPI loopback đều ổn định.',
  'Độ trễ tổng từ đầu đến cuối hiện đang khoảng hai trăm sáu mươi mili-giây.',
  'Con số đó thấp hơn nhiều so với mục tiêu dưới một giây, nên chúng ta đang ở vị thế tốt.',
  'Cột mốc tiếp theo là phát hành bản nguyên mẫu vào cuối tuần.',
];

function OverlayWindow({ width = 600, state = 'active', lines = 5, hover = false }) {
  const shown = OVERLAY_LINES.slice(-lines);
  const opacityFor = (i) => shown.length === 1 ? 1 : 0.35 + (0.65 * i) / (shown.length - 1);
  // height: 14px (top breathing space) + lines * 28 + 16 (bottom padding) — strip is overlaid
  const height = 14 + lines * 28 + 16;

  return (
    <div style={{
      width, height, borderRadius: 20,
      background: V.ovBg,
      backdropFilter: 'blur(40px) saturate(160%)',
      WebkitBackdropFilter: 'blur(40px) saturate(160%)',
      border: `0.5px solid ${V.ovBd}`,
      boxShadow: V.ovShadow,
      color: V.text, fontFamily: V.font,
      position: 'relative', overflow: 'hidden',
    }}>
      {/* Hover strip — 24px, slides over the top edge */}
      <div style={{
        position: 'absolute', top: 0, left: 0, right: 0, height: 24,
        display: 'flex', alignItems: 'center', gap: 8, padding: '0 12px',
        background: 'linear-gradient(to bottom, var(--vt-bg-deep), transparent)',
        opacity: hover ? 1 : 0,
        transform: hover ? 'translateY(0)' : 'translateY(-4px)',
        transition: 'opacity .12s, transform .12s',
        pointerEvents: hover ? 'auto' : 'none',
        zIndex: 3,
      }}>
        {state === 'active' && <div style={{
          width: 7, height: 7, borderRadius: '50%', background: V.rec,
          boxShadow: `0 0 0 3px ${V.recGlow}`,
        }} />}
        {state === 'reconnecting' && <div style={{
          width: 7, height: 7, borderRadius: '50%', background: V.warn,
        }} />}
        {state === 'idle' && <div style={{
          width: 7, height: 7, borderRadius: '50%', background: V.dim,
        }} />}
        <span style={{
          fontFamily: V.mono, fontSize: 9, color: V.subtle, letterSpacing: 0.5,
        }}>
          {state === 'reconnecting' ? 'RECONNECTING' : state === 'idle' ? 'IDLE' : 'EN → VI'}
        </span>
        {/* drag region (grab cursor) */}
        <div style={{ flex: 1, height: 24, cursor: 'grab' }} />
        <button title="Close overlay" style={{
          width: 22, height: 22, borderRadius: 6,
          background: 'transparent', border: 'none', color: V.muted, cursor: 'pointer',
          display: 'inline-flex', alignItems: 'center', justifyContent: 'center',
        }}>
          <Icon d={Icons.close} size={12} />
        </button>
      </div>
      {/* Text stack — fills the whole window in default state */}
      <div style={{
        position: 'absolute', inset: 0,
        padding: '14px 18px 16px',
        display: 'flex', flexDirection: 'column', justifyContent: 'flex-end',
        gap: 2, minWidth: 0,
      }}>
        {state === 'active' && shown.map((text, i) => {
          const isLast = i === shown.length - 1;
          return (
            <div key={i} style={{
              fontSize: isLast ? 17 : 14,
              fontWeight: isLast ? 500 : 400,
              lineHeight: 1.3, letterSpacing: -0.15,
              color: V.text, opacity: opacityFor(i),
              whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis',
            }}>
              {text}
              {isLast && <span style={{
                display: 'inline-block', width: 7, height: 15, marginLeft: 3,
                background: V.accent, verticalAlign: -2,
                animation: 'vt-blink 0.9s steps(2) infinite',
              }} />}
            </div>
          );
        })}
        {state === 'reconnecting' && (
          <>
            <div style={{ fontSize: 12, color: V.dim, marginBottom: 4 }}>
              Connection to Soniox dropped — exponential backoff (attempt 2 of 6, retrying in 1 s)
            </div>
            <div style={{
              fontSize: 17, fontWeight: 500, color: V.warn, letterSpacing: -0.15,
            }}>Đang kết nối lại…</div>
          </>
        )}
        {state === 'idle' && (
          <>
            <div style={{ fontSize: 12, color: V.dim, marginBottom: 4 }}>
              Voxtide overlay · open the main window and press <span style={{
                fontFamily: V.mono, color: V.muted,
              }}>⌃⇧V</span> to start
            </div>
            <div style={{
              fontSize: 17, fontWeight: 500, color: V.muted, letterSpacing: -0.15,
            }}>Waiting for audio</div>
          </>
        )}
      </div>
    </div>
  );
}

// ─── settings sheet ─────────────────────────────────────────────────────────

function SettingsSection({ title, subtitle, last, children }) {
  return (
    <div style={{
      paddingBottom: last ? 0 : 18, marginBottom: last ? 0 : 18,
      borderBottom: last ? 'none' : `0.5px solid ${V.border}`,
    }}>
      <div style={{
        fontSize: 12, fontWeight: 600, color: V.text, letterSpacing: -0.1,
        marginBottom: subtitle ? 4 : 10,
      }}>{title}</div>
      {subtitle && <div style={{
        fontSize: 11.5, color: V.subtle, marginBottom: 10, lineHeight: 1.5,
      }}>{subtitle}</div>}
      {children}
    </div>
  );
}
function Row({ label, value, icon }) {
  return (
    <div style={{
      display: 'flex', alignItems: 'center', padding: '10px 12px',
      background: V.surface, borderRadius: 8, marginBottom: 6,
      border: `0.5px solid ${V.border}`, fontSize: 12,
    }}>
      {icon && <div style={{ marginRight: 10, color: V.muted, display: 'flex' }}>
        <Icon d={icon} size={14} />
      </div>}
      <span style={{ color: V.text }}>{label}</span>
      <div style={{ flex: 1 }} />
      <span style={{ color: V.muted, display: 'flex', alignItems: 'center', gap: 6 }}>{value}</span>
      <Icon d={Icons.chevron} size={12} color={V.subtle} />
    </div>
  );
}
function LangCard({ code, name, sub, mine, onSelect }) {
  return (
    <button onClick={onSelect} style={{
      flex: 1, padding: 12, borderRadius: 10,
      background: mine ? V.accent10 : V.surface,
      border: `0.5px solid ${mine ? V.accent50 : V.border}`,
      color: V.text, fontFamily: V.font, cursor: 'pointer',
      textAlign: 'left', position: 'relative',
    }}>
      <div style={{
        fontFamily: V.mono, fontSize: 10, color: mine ? V.accent : V.subtle,
        letterSpacing: 0.6, marginBottom: 4,
      }}>{sub.toUpperCase()}</div>
      <div style={{ display: 'flex', alignItems: 'baseline', gap: 8 }}>
        <span style={{
          fontFamily: V.mono, fontSize: 18, fontWeight: 600, letterSpacing: 0.4,
          color: V.text,
        }}>{code}</span>
        <span style={{ fontSize: 13, color: V.muted }}>{name}</span>
      </div>
      {mine && <div style={{
        position: 'absolute', top: 10, right: 10,
        background: V.accent, color: V.accentInk,
        fontSize: 9, fontWeight: 700, padding: '2px 6px', borderRadius: 4,
        letterSpacing: 0.4,
      }}>MY LANGUAGE</div>}
    </button>
  );
}

function SettingsSheet() {
  const [theme, setTheme] = React.useState('dark');
  const [mine, setMine] = React.useState('a');
  return (
    <div style={{
      width: 560, height: 680, borderRadius: 14,
      background: V.bg, color: V.text, fontFamily: V.font,
      border: `0.5px solid ${V.border}`,
      boxShadow: V.winShadow,
      display: 'flex', flexDirection: 'column', overflow: 'hidden',
    }}>
      <div style={{
        height: 44, padding: '0 16px', display: 'flex', alignItems: 'center', gap: 10,
        borderBottom: `0.5px solid ${V.border}`,
      }}>
        <TrafficLights />
        <div style={{ flex: 1, textAlign: 'center', fontSize: 13, fontWeight: 600 }}>
          Settings
        </div>
        <div style={{ width: 50 }} />
      </div>
      <div style={{ flex: 1, overflow: 'auto', padding: '20px 24px 24px' }}>
        <SettingsSection title="Soniox API key" subtitle="Stored in OS keychain. Never re-shown after save.">
          <div style={{
            display: 'flex', alignItems: 'center', gap: 8,
            padding: '8px 10px', borderRadius: 8,
            background: V.surface, border: `0.5px solid ${V.border}`,
          }}>
            <Icon d={Icons.key} size={13} color={V.muted} />
            <span style={{
              flex: 1, fontFamily: V.mono, fontSize: 12, color: V.muted, letterSpacing: 1,
            }}>••••••••••••••••••••••••••••••••</span>
            <span style={{ fontSize: 11, color: V.subtle }}>Last updated May 4</span>
            <button style={{
              padding: '4px 9px', borderRadius: 5, cursor: 'pointer',
              background: V.surface3, color: V.text, border: `0.5px solid ${V.borderHi}`,
              fontFamily: V.font, fontSize: 11, fontWeight: 500,
            }}>Replace</button>
          </div>
        </SettingsSection>

        <SettingsSection title="Default languages"
          subtitle="Restored on next launch. In Meeting mode, the language marked as “mine” becomes the translation target.">
          <div style={{ display: 'flex', gap: 8, marginBottom: 10 }}>
            <LangCard code="EN" name="English" sub="Language A"
                      mine={mine === 'a'} onSelect={() => setMine('a')} />
            <LangCard code="VI" name="Vietnamese" sub="Language B"
                      mine={mine === 'b'} onSelect={() => setMine('b')} />
          </div>
          <div style={{ fontSize: 11, color: V.subtle, lineHeight: 1.5 }}>
            Click a language card to mark it as <span style={{ color: V.text }}>your language</span>.
          </div>
        </SettingsSection>

        <SettingsSection title="Default audio source per mode">
          <Row label="Meeting (system audio)" value="Default output — Zoom meeting" icon={Icons.speaker} />
          <Row label="Conversation (microphone)" value="MacBook Pro Microphone" icon={Icons.mic} />
        </SettingsSection>

        <SettingsSection title="Global hotkey">
          <Row label="Start / Stop" value={<span style={{ display: 'flex', gap: 4 }}>
            <kbd style={kbd()}>⌃</kbd><kbd style={kbd()}>⇧</kbd><kbd style={kbd()}>V</kbd>
          </span>} />
        </SettingsSection>

        <SettingsSection title="Appearance" last>
          <div style={{
            display: 'inline-flex', padding: 2, background: V.surface,
            borderRadius: 8, border: `0.5px solid ${V.border}`,
          }}>
            {['light', 'dark', 'system'].map(t => (
              <button key={t} onClick={() => setTheme(t)} style={{
                padding: '6px 14px', borderRadius: 6, border: 'none', cursor: 'pointer',
                background: theme === t ? V.surface3 : 'transparent',
                color: theme === t ? V.text : V.muted,
                fontFamily: V.font, fontSize: 12, fontWeight: 500, textTransform: 'capitalize',
              }}>{t}</button>
            ))}
          </div>
        </SettingsSection>
      </div>
    </div>
  );
}

// ─── app icon system ────────────────────────────────────────────────────────

function AppIconSquircle({ size = 1024, bars = 5 }) {
  // squircle (22% radius), cyan gradient bg, white wave glyph, inner highlight + shadow
  const r = size * 0.22;
  const stroke = Math.max(0.5, size * 0.0015);
  return (
    <div style={{
      width: size, height: size, borderRadius: r, position: 'relative',
      background: `linear-gradient(155deg, oklch(0.78 0.14 200), oklch(0.50 0.16 220))`,
      boxShadow: `inset 0 ${size * 0.02}px ${size * 0.04}px rgba(255,255,255,0.35),
                  inset 0 -${size * 0.03}px ${size * 0.06}px rgba(0,0,0,0.25),
                  inset 0 0 0 ${stroke}px rgba(255,255,255,0.2),
                  0 ${size * 0.02}px ${size * 0.05}px rgba(0,0,0,0.25)`,
      overflow: 'hidden',
    }}>
      <div style={{
        position: 'absolute', inset: 0,
        background: `radial-gradient(circle at 30% 25%, rgba(255,255,255,0.20), transparent 55%)`,
      }} />
      <div style={{
        position: 'absolute', inset: 0,
        display: 'flex', alignItems: 'center', justifyContent: 'center',
      }}>
        <WaveGlyph size={size * 0.55} color="#fff" bars={bars} />
      </div>
    </div>
  );
}

function AppIconV({ size = 16 }) {
  // single "V" mark using two intersecting wave-curves — survives at tiny sizes
  const r = size * 0.22;
  return (
    <div style={{
      width: size, height: size, borderRadius: r,
      background: 'linear-gradient(155deg, oklch(0.78 0.14 200), oklch(0.50 0.16 220))',
      display: 'flex', alignItems: 'center', justifyContent: 'center',
      boxShadow: 'inset 0 0.5px 0 rgba(255,255,255,0.4)',
    }}>
      <svg width={size * 0.7} height={size * 0.7} viewBox="0 0 24 24" fill="none">
        <path d="M4 6 Q 8 22, 12 22 Q 16 22, 20 6"
              stroke="#fff" strokeWidth="3.2" strokeLinecap="round" fill="none" />
      </svg>
    </div>
  );
}

function AppIconMenuBar({ size = 22 }) {
  // monochrome template: just the V silhouette, no rounded square
  return (
    <svg width={size} height={size} viewBox="0 0 22 22" fill="none">
      <path d="M3 6 Q 7 19, 11 19 Q 15 19, 19 6"
            stroke="currentColor" strokeWidth="2.4" strokeLinecap="round" fill="none" />
      <path d="M6 6 Q 8 13, 11 13 Q 14 13, 16 6"
            stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" fill="none" opacity="0.55" />
    </svg>
  );
}

// ─── desktop backdrop ───────────────────────────────────────────────────────

function Desktop({ children, height = 700, theme = 'dark', style = {} }) {
  const dark = 'linear-gradient(140deg, oklch(0.30 0.04 230) 0%, oklch(0.20 0.03 260) 50%, oklch(0.16 0.02 280) 100%)';
  const light = 'linear-gradient(140deg, oklch(0.85 0.06 220) 0%, oklch(0.92 0.04 240) 50%, oklch(0.88 0.05 280) 100%)';
  return (
    <div style={{
      width: '100%', height, position: 'relative', overflow: 'hidden',
      background: theme === 'light' ? light : dark,
      display: 'flex', alignItems: 'center', justifyContent: 'center',
      ...style,
    }}>
      <div style={{
        position: 'absolute', inset: 0,
        backgroundImage: theme === 'light'
          ? 'radial-gradient(rgba(255,255,255,0.30) 1px, transparent 1px)'
          : 'radial-gradient(rgba(255,255,255,0.06) 1px, transparent 1px)',
        backgroundSize: '20px 20px', opacity: 0.4,
      }} />
      {children}
    </div>
  );
}

// Theme wrapper — must wrap any of the components above to set CSS vars.
function Themed({ theme = 'dark', children, style = {} }) {
  return (
    <div className={theme === 'light' ? 'vt-theme-light' : 'vt-theme-dark'}
         style={{ ...style }}>
      {children}
    </div>
  );
}

Object.assign(window, {
  MainWindow, OverlayWindow, SettingsSheet, Desktop, Themed,
  AppIconSquircle, AppIconV, AppIconMenuBar, WaveGlyph,
  StatusBar, V, kbd,
});

// Voxtide UI — main window, overlay window, settings, onboarding
// One file, all the screens for the design canvas.

const V = {
  font: '"Geist", -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
  mono: '"Geist Mono", "JetBrains Mono", ui-monospace, monospace',
  bg:        'oklch(0.16 0.005 250)',
  bgDeep:    'oklch(0.12 0.005 250)',
  surface:   'oklch(0.20 0.006 250)',
  surface2:  'oklch(0.24 0.007 250)',
  surface3:  'oklch(0.28 0.008 250)',
  border:    'oklch(0.30 0.008 250)',
  borderHi:  'oklch(0.40 0.010 250)',
  text:      'oklch(0.97 0.003 250)',
  muted:     'oklch(0.70 0.005 250)',
  subtle:    'oklch(0.52 0.005 250)',
  dim:       'oklch(0.38 0.005 250)',
  accent:    'oklch(0.80 0.13 205)',
  accentDim: 'oklch(0.42 0.08 205)',
  accentInk: 'oklch(0.18 0.04 205)',
  rec:       'oklch(0.72 0.20 25)',
  warn:      'oklch(0.82 0.14 80)',
  ok:        'oklch(0.78 0.13 155)',
};

// ─── primitives ─────────────────────────────────────────────────────────────

const TrafficLights = ({ dim = false }) => (
  <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
    {['#ff5f57', '#febc2e', '#28c840'].map(c => (
      <div key={c} style={{
        width: 12, height: 12, borderRadius: '50%',
        background: dim ? 'oklch(0.30 0.005 250)' : c,
        boxShadow: 'inset 0 0 0 0.5px rgba(0,0,0,0.25)',
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
  stop:     "M6 6h12v12H6z",
  arrows:   "M7 7l-3 3 3 3 M4 10h14 M17 17l3-3-3-3 M20 14H6",
  swap:     "M7 7h13l-3-3 M17 17H4l3 3",
  chevron:  "M6 9l6 6 6-6",
  pin:      "M12 3l3 5 5 1-4 4 1 5-5-3-5 3 1-5-4-4 5-1z",
  close:    "M6 6l12 12 M18 6L6 18",
  plus:     "M12 5v14 M5 12h14",
  check:    "M5 12l5 5 9-12",
  drag:     "M9 4h2v2H9z M13 4h2v2h-2z M9 10h2v2H9z M13 10h2v2h-2z M9 16h2v2H9z M13 16h2v2h-2z",
  desktop:  "M3 5h18v12H3z M9 21h6 M12 17v4",
  globe:    "M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18z M3 12h18 M12 3a14 14 0 0 1 0 18 M12 3a14 14 0 0 0 0 18",
  key:      "M14 8a4 4 0 1 1-4 4 M14 8l6 0 m0 0v3 m-3-3v3 M10 12L3 19v2h2l1-1h2v-2h2l1-1",
};

// ─── window chrome ──────────────────────────────────────────────────────────

function VoxWindow({ width = 920, height = 600, children, focused = true }) {
  return (
    <div style={{
      width, height, borderRadius: 12, overflow: 'hidden',
      background: V.bg, color: V.text, fontFamily: V.font,
      boxShadow: focused
        ? '0 0 0 0.5px rgba(0,0,0,0.6), 0 1px 0 rgba(255,255,255,0.04) inset, 0 30px 60px -20px rgba(0,0,0,0.6), 0 18px 36px -18px rgba(0,0,0,0.5)'
        : '0 0 0 0.5px rgba(0,0,0,0.5), 0 12px 24px rgba(0,0,0,0.3)',
      display: 'flex', flexDirection: 'column',
      border: `0.5px solid ${V.border}`,
    }}>
      {children}
    </div>
  );
}

// ─── top toolbar ────────────────────────────────────────────────────────────

function ModeToggle({ mode, onChange }) {
  return (
    <div style={{
      display: 'inline-flex', padding: 2, background: V.surface,
      borderRadius: 8, border: `0.5px solid ${V.border}`,
      fontSize: 12, fontWeight: 500, letterSpacing: -0.1,
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
            boxShadow: active ? `0 0 0 0.5px ${V.borderHi}, 0 1px 0 rgba(255,255,255,0.04) inset` : 'none',
            fontFamily: V.font, fontSize: 12, fontWeight: 500,
            transition: 'all .15s',
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

function IconBtn({ d, active, onClick, size = 30, title }) {
  return (
    <button onClick={onClick} title={title} style={{
      width: size, height: size, borderRadius: 7,
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
      boxShadow: '0 1px 0 rgba(255,255,255,0.15) inset, 0 6px 14px -6px rgba(0,0,0,0.6)',
    }}>
      {recording
        ? <div style={{ width: 8, height: 8, borderRadius: 2, background: '#fff' }} />
        : <Icon d={Icons.play} size={11} />}
      {children}
    </button>
  );
}

function Toolbar({ mode, setMode, recording, setRecording, draggable = true }) {
  const langs = {
    meeting: { a: { code: 'EN', name: 'English' }, b: { code: 'VI', name: 'Vietnamese' }, mine: 'b' },
    conversation: { a: { code: 'EN', name: 'English' }, b: { code: 'JA', name: 'Japanese' }, mine: 'a' },
  }[mode];
  return (
    <div style={{
      height: 48, display: 'flex', alignItems: 'center', gap: 10,
      padding: '0 12px 0 14px',
      borderBottom: `0.5px solid ${V.border}`,
      background: V.bg,
      WebkitAppRegion: draggable ? 'drag' : 'no-drag',
    }}>
      <TrafficLights />
      <div style={{
        width: 1, height: 18, background: V.border, marginLeft: 4, marginRight: 4,
      }} />
      {/* Wordmark */}
      <div style={{
        display: 'flex', alignItems: 'center', gap: 7,
      }}>
        <div style={{
          width: 18, height: 18, borderRadius: 5,
          background: `linear-gradient(135deg, ${V.accent}, ${V.accentDim})`,
          display: 'flex', alignItems: 'center', justifyContent: 'center',
          boxShadow: `0 0 0 0.5px ${V.accentDim}`,
        }}>
          {/* Sound wave glyph */}
          <svg width="11" height="11" viewBox="0 0 12 12" fill="none">
            <path d="M2 6h1 M4 4v4 M6 2v8 M8 4v4 M10 6h0" stroke={V.accentInk} strokeWidth="1.4" strokeLinecap="round" />
          </svg>
        </div>
        <span style={{
          fontFamily: V.font, fontSize: 13, fontWeight: 600, letterSpacing: -0.2,
          color: V.text,
        }}>Voxtide</span>
      </div>
      <div style={{ width: 1, height: 18, background: V.border, marginLeft: 4, marginRight: 6 }} />
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

// ─── sidebar ────────────────────────────────────────────────────────────────

const SESSIONS = [
  { date: 'Today · 14:22', dur: '38m', a: 'EN', b: 'VI', mode: 'meeting',
    preview: 'So the next milestone is to ship the prototype by end of week…', active: true },
  { date: 'Today · 09:05', dur: '12m', a: 'EN', b: 'JA', mode: 'conversation',
    preview: 'すみません、駅はどこですか — Excuse me, where is the station?' },
  { date: 'Yesterday', dur: '1h 04m', a: 'EN', b: 'VI', mode: 'meeting',
    preview: 'Quarterly review with the Hanoi team. Q3 targets revisited and…' },
  { date: 'May 10', dur: '22m', a: 'KO', b: 'EN', mode: 'meeting',
    preview: '안녕하세요 — Hi everyone, thanks for joining today\u2019s sync.' },
  { date: 'May 9', dur: '7m', a: 'EN', b: 'ES', mode: 'conversation',
    preview: '¿Dónde está el baño? — Where is the restroom?' },
  { date: 'May 8', dur: '46m', a: 'EN', b: 'VI', mode: 'meeting',
    preview: 'Architecture review — Tauri vs Electron tradeoffs, audio routing.' },
];

function Sidebar({ activeIdx = 0, query = '' }) {
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
          <input defaultValue={query} placeholder="Search transcripts…" style={{
            flex: 1, background: 'transparent', border: 'none', outline: 'none',
            color: V.text, fontFamily: V.font, fontSize: 12,
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
          fontFamily: V.font, fontSize: 10, fontWeight: 600, letterSpacing: 0.6,
          color: V.subtle, textTransform: 'uppercase',
        }}>History</span>
        <button style={{
          width: 18, height: 18, borderRadius: 4, background: 'transparent',
          border: 'none', color: V.subtle, cursor: 'pointer',
          display: 'inline-flex', alignItems: 'center', justifyContent: 'center',
        }}><Icon d={Icons.plus} size={12} /></button>
      </div>
      <div style={{ flex: 1, overflow: 'auto', padding: '0 8px 8px' }}>
        {SESSIONS.map((s, i) => {
          const active = i === activeIdx;
          return (
            <div key={i} style={{
              padding: '8px 9px', borderRadius: 7, marginBottom: 2,
              background: active ? V.surface2 : 'transparent',
              border: `0.5px solid ${active ? V.border : 'transparent'}`,
              cursor: 'pointer',
            }}>
              <div style={{
                display: 'flex', alignItems: 'center', justifyContent: 'space-between',
                marginBottom: 4,
              }}>
                <span style={{ fontSize: 11, color: V.muted }}>{s.date}</span>
                <span style={{
                  fontFamily: V.mono, fontSize: 10, color: V.subtle,
                }}>{s.dur}</span>
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: 5, marginBottom: 5 }}>
                <span style={LangTag()}>{s.a}</span>
                <Icon d="M5 12h14 M14 7l5 5-5 5" size={10} color={V.subtle} />
                <span style={LangTag()}>{s.b}</span>
                <span style={{
                  marginLeft: 'auto', fontSize: 9.5, color: V.subtle,
                  textTransform: 'uppercase', letterSpacing: 0.5,
                }}>{s.mode}</span>
                {s.active && <div style={{
                  width: 6, height: 6, borderRadius: '50%', background: V.rec,
                  boxShadow: `0 0 0 2px oklch(0.72 0.20 25 / 0.25)`,
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
    </div>
  );
}

const LangTag = () => ({
  fontFamily: V.mono, fontSize: 9.5, fontWeight: 600, color: V.muted,
  background: V.surface, padding: '1px 4px', borderRadius: 3,
  letterSpacing: 0.4, border: `0.5px solid ${V.border}`,
});

// ─── transcript ─────────────────────────────────────────────────────────────

const MEETING_TRANSCRIPT = [
  { o: "Alright, so let's start with the engineering update.",
    t: "Được rồi, hãy bắt đầu với cập nhật từ đội kỹ thuật.", t_ms: '14:22:08' },
  { o: "We're tracking ahead on the audio capture layer, both macOS taps and WASAPI loopback are stable.",
    t: "Chúng tôi đang đi trước tiến độ ở lớp thu âm thanh, cả Core Audio taps trên macOS và WASAPI loopback đều đã ổn định.", t_ms: '14:22:14' },
  { o: "Latency end-to-end is hovering around two hundred sixty milliseconds.",
    t: "Độ trễ tổng từ đầu đến cuối hiện đang khoảng hai trăm sáu mươi mili-giây.", t_ms: '14:22:23' },
  { o: "That's well under our sub-second target, so we're comfortable there.",
    t: "Con số đó thấp hơn nhiều so với mục tiêu dưới một giây, nên chúng ta đang ở vị thế tốt.", t_ms: '14:22:31' },
  { o: "Next milestone is to ship the prototype by end of week and start internal dogfooding on Monday.",
    t: "Cột mốc tiếp theo là phát hành bản nguyên mẫu vào cuối tuần và bắt đầu dùng nội bộ từ thứ Hai.", t_ms: '14:22:40', live: true },
];

const CONV_TRANSCRIPT = [
  { speaker: 'A', a: 'EN', b: 'JA',
    o: "Excuse me, do you know the way to Shibuya station?",
    t: "すみません、渋谷駅への道をご存じですか？", t_ms: '09:05:12' },
  { speaker: 'B', a: 'JA', b: 'EN',
    o: "はい、この道をまっすぐ行って、二つ目の信号を右に曲がってください。",
    t: "Yes — go straight down this road, then turn right at the second traffic light.", t_ms: '09:05:18' },
  { speaker: 'A', a: 'EN', b: 'JA',
    o: "Thank you — about how many minutes is it on foot?",
    t: "ありがとうございます — 歩いて何分くらいかかりますか？", t_ms: '09:05:26' },
  { speaker: 'B', a: 'JA', b: 'EN',
    o: "そうですね、十分くらいだと思います。",
    t: "Let me think — about ten minutes, I'd say.", t_ms: '09:05:33', live: true },
];

function LevelMeter({ active }) {
  // 12-bar mic-level meter — animated when active
  const [tick, setTick] = React.useState(0);
  React.useEffect(() => {
    if (!active) return;
    const id = setInterval(() => setTick(t => t + 1), 80);
    return () => clearInterval(id);
  }, [active]);
  const bars = Array.from({ length: 14 }, (_, i) => {
    if (!active) return 0.12;
    // pseudo-random but stable-ish wave
    const v = 0.25 + 0.5 * Math.abs(Math.sin((tick + i * 1.3) * 0.6 + i));
    return Math.min(1, v * (0.6 + 0.4 * Math.sin(tick * 0.2 + i * 0.4)));
  });
  return (
    <div style={{ display: 'inline-flex', alignItems: 'center', gap: 2, height: 14 }}>
      {bars.map((h, i) => (
        <div key={i} style={{
          width: 2, height: `${100 * h}%`, minHeight: 2,
          borderRadius: 1,
          background: active
            ? (h > 0.7 ? V.accent : V.accentDim)
            : V.dim,
          transition: 'height .08s',
        }} />
      ))}
    </div>
  );
}

function StatusBar({ recording, mode, elapsed = '00:38:24' }) {
  return (
    <div style={{
      height: 28, borderTop: `0.5px solid ${V.border}`, background: V.bgDeep,
      display: 'flex', alignItems: 'center', gap: 14,
      padding: '0 12px',
      fontFamily: V.mono, fontSize: 10.5, color: V.subtle, letterSpacing: 0.2,
    }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
        <div style={{
          width: 7, height: 7, borderRadius: '50%',
          background: recording ? V.rec : V.dim,
          boxShadow: recording ? `0 0 0 3px oklch(0.72 0.20 25 / 0.15)` : 'none',
        }} />
        <span style={{ color: recording ? V.text : V.subtle, textTransform: 'uppercase' }}>
          {recording ? 'REC' : 'IDLE'}
        </span>
        <span>{recording ? elapsed : '—'}</span>
      </div>
      <span style={{ color: V.dim }}>│</span>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        <LevelMeter active={recording} />
        <span>{recording ? '-18 dB' : ''}</span>
      </div>
      <span style={{ color: V.dim }}>│</span>
      <span>SONIOX · stt-rt-v4</span>
      <span style={{ color: V.dim }}>│</span>
      <span>{recording ? '262 ms' : 'ws idle'}</span>
      <div style={{ flex: 1 }} />
      <span>{mode === 'meeting' ? 'one_way → VI' : 'two_way · EN ⇄ JA'}</span>
      <span style={{ color: V.dim }}>│</span>
      <span>16 kHz · mono · s16le</span>
    </div>
  );
}

// ─── transcript pane ────────────────────────────────────────────────────────

function MeetingTranscript({ recording }) {
  // animate live ghost text
  const finalLine = "Next milestone is to ship the prototype by end of week and start internal dogfooding on Monday.";
  const finalTrans = "Cột mốc tiếp theo là phát hành bản nguyên mẫu vào cuối tuần và bắt đầu dùng nội bộ từ thứ Hai.";
  const [n, setN] = React.useState(0);
  React.useEffect(() => {
    if (!recording) return;
    const id = setInterval(() => setN(x => (x + 1) % 130), 90);
    return () => clearInterval(id);
  }, [recording]);
  const live = recording;
  const oLive = live ? finalLine.slice(0, Math.min(finalLine.length, Math.floor(n * 0.7))) : '';
  const tLive = live ? finalTrans.slice(0, Math.min(finalTrans.length, Math.max(0, Math.floor((n - 25) * 0.7)))) : '';

  return (
    <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
      {/* Original column */}
      <Column
        label="Original" code="EN" sub="auto-detected"
        body={MEETING_TRANSCRIPT.slice(0, -1).map((l, i) => (
          <Line key={i} ts={l.t_ms} text={l.o} />
        ))}
        ghost={oLive ? <Line live ts="now" text={oLive} /> : null}
      />
      <div style={{ width: 0.5, background: V.border }} />
      {/* Translation column */}
      <Column
        label="Translation" code="VI" sub="Vietnamese — target" accent
        body={MEETING_TRANSCRIPT.slice(0, -1).map((l, i) => (
          <Line key={i} ts={l.t_ms} text={l.t} translated />
        ))}
        ghost={tLive ? <Line live ts="now" text={tLive} translated /> : null}
      />
    </div>
  );
}

function ConversationTranscript({ recording }) {
  return (
    <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
      <Column
        label="Original" code="EN/JA" sub="auto-detected per turn"
        body={CONV_TRANSCRIPT.map((l, i) => (
          <Line key={i} ts={l.t_ms} text={l.o} speaker={l.speaker} speakerLang={l.a}
                live={l.live && recording} />
        ))}
      />
      <div style={{ width: 0.5, background: V.border }} />
      <Column
        label="Translation" code="EN ⇄ JA" sub="two-way" accent
        body={CONV_TRANSCRIPT.map((l, i) => (
          <Line key={i} ts={l.t_ms} text={l.t} translated speaker={l.speaker} speakerLang={l.b}
                live={l.live && recording} />
        ))}
      />
    </div>
  );
}

function Column({ label, code, sub, body, ghost, accent }) {
  return (
    <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0 }}>
      <div style={{
        height: 38, padding: '0 16px', display: 'flex', alignItems: 'center', gap: 8,
        borderBottom: `0.5px solid ${V.border}`,
        background: V.bg,
      }}>
        <span style={{
          fontSize: 11, fontWeight: 600, color: V.text, letterSpacing: -0.1,
        }}>{label}</span>
        <span style={{
          fontFamily: V.mono, fontSize: 9.5, fontWeight: 600, letterSpacing: 0.4,
          color: accent ? V.accent : V.muted,
          background: accent ? 'oklch(0.80 0.13 205 / 0.10)' : V.surface,
          padding: '2px 5px', borderRadius: 3,
          border: `0.5px solid ${accent ? 'oklch(0.80 0.13 205 / 0.25)' : V.border}`,
        }}>{code}</span>
        <span style={{ fontSize: 11, color: V.subtle }}>{sub}</span>
      </div>
      <div style={{ flex: 1, overflow: 'auto', padding: '4px 0 12px' }}>
        {body}
        {ghost}
      </div>
    </div>
  );
}

function Line({ ts, text, translated, speaker, speakerLang, live }) {
  return (
    <div style={{
      padding: '10px 16px 10px 16px',
      display: 'grid',
      gridTemplateColumns: '52px 1fr',
      gap: 10,
      borderTop: `0.5px solid oklch(0.30 0.008 250 / 0.5)`,
    }}>
      <div style={{
        fontFamily: V.mono, fontSize: 10, color: V.dim, letterSpacing: 0.3,
        paddingTop: 3,
      }}>{ts}</div>
      <div>
        {speaker && (
          <div style={{
            display: 'inline-flex', alignItems: 'center', gap: 6, marginBottom: 5,
          }}>
            <div style={{
              width: 16, height: 16, borderRadius: 4,
              background: speaker === 'A' ? V.accent : 'oklch(0.78 0.13 25)',
              color: V.accentInk,
              fontFamily: V.font, fontSize: 9.5, fontWeight: 700,
              display: 'inline-flex', alignItems: 'center', justifyContent: 'center',
              letterSpacing: 0,
            }}>{speaker}</div>
            <span style={{
              fontFamily: V.mono, fontSize: 9.5, color: V.subtle, letterSpacing: 0.4,
            }}>{speakerLang}</span>
          </div>
        )}
        <div style={{
          fontSize: 13.5, lineHeight: 1.55,
          color: live ? V.muted : (translated ? V.text : V.text),
          fontWeight: translated ? 400 : 400,
          fontStyle: live ? 'normal' : 'normal',
        }}>
          {text}
          {live && <span style={{
            display: 'inline-block', width: 8, height: 14, marginLeft: 2,
            background: translated ? V.accent : V.muted,
            verticalAlign: -2,
            animation: 'vt-blink 0.9s steps(2) infinite',
          }} />}
        </div>
      </div>
    </div>
  );
}

// ─── empty / onboarding states ──────────────────────────────────────────────

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
      <div style={{
        fontSize: 15, fontWeight: 600, color: V.text, letterSpacing: -0.2,
      }}>
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
const kbd = () => ({
  fontFamily: V.mono, fontSize: 10.5, padding: '2px 6px',
  borderRadius: 4, background: V.surface, border: `0.5px solid ${V.border}`,
  color: V.muted,
});

function NoApiKey() {
  return (
    <div style={{
      flex: 1, display: 'flex', flexDirection: 'column',
      alignItems: 'center', justifyContent: 'center', padding: 24, gap: 12,
    }}>
      <div style={{
        width: 56, height: 56, borderRadius: 12,
        background: 'oklch(0.82 0.14 80 / 0.10)',
        border: `0.5px solid oklch(0.82 0.14 80 / 0.4)`,
        color: V.warn,
        display: 'flex', alignItems: 'center', justifyContent: 'center',
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
        <button style={btnPrimary()}>
          Add API key
        </button>
        <button style={btnGhost()}>
          Get a Soniox key →
        </button>
      </div>
    </div>
  );
}
const btnPrimary = () => ({
  padding: '8px 14px', borderRadius: 7, border: 'none', cursor: 'pointer',
  background: V.accent, color: V.accentInk,
  fontFamily: V.font, fontSize: 12, fontWeight: 600,
});
const btnGhost = () => ({
  padding: '8px 14px', borderRadius: 7, cursor: 'pointer',
  background: 'transparent', color: V.muted,
  border: `0.5px solid ${V.border}`,
  fontFamily: V.font, fontSize: 12, fontWeight: 500,
});

// ─── MAIN window screen ─────────────────────────────────────────────────────

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
          <StatusBar recording={recording} mode={mode} />
        </div>
      </div>
    </VoxWindow>
  );
}

// ─── overlay window ─────────────────────────────────────────────────────────

const OVERLAY_LINES = [
  'Được rồi, hãy bắt đầu với cập nhật từ đội kỹ thuật.',
  'Chúng tôi đang đi trước tiến độ ở lớp thu âm thanh — cả Core Audio taps và WASAPI loopback đều ổn định.',
  'Độ trễ tổng từ đầu đến cuối hiện đang khoảng hai trăm sáu mươi mili-giây.',
  'Con số đó thấp hơn nhiều so với mục tiêu dưới một giây, nên chúng ta đang ở vị thế tốt.',
  'Cột mốc tiếp theo là phát hành bản nguyên mẫu vào cuối tuần.',
];

function OverlayWindow({ width = 600, state = 'active', oneLine = false, lines = 5 }) {
  // states: 'active', 'idle', 'reconnecting'
  if (oneLine) lines = 1;
  const shown = OVERLAY_LINES.slice(-lines);
  // Older lines fade out (opacity ramps up to newest)
  const opacityFor = (i) => {
    if (shown.length === 1) return 1;
    return 0.35 + (0.65 * i) / (shown.length - 1);
  };
  const height = oneLine ? 60 : 18 + lines * 28 + 22; // handle + lines + footer pad

  return (
    <div style={{
      width, height, borderRadius: 20,
      background: 'oklch(0.13 0.005 250 / 0.88)',
      backdropFilter: 'blur(40px) saturate(150%)',
      WebkitBackdropFilter: 'blur(40px) saturate(150%)',
      border: `0.5px solid oklch(0.50 0.010 250 / 0.5)`,
      boxShadow: '0 1px 0 rgba(255,255,255,0.06) inset, 0 20px 50px -10px rgba(0,0,0,0.5), 0 8px 20px -8px rgba(0,0,0,0.6)',
      color: V.text, fontFamily: V.font,
      display: 'flex', flexDirection: 'column',
      position: 'relative', overflow: 'hidden',
    }}>
      {/* Drag handle */}
      <div style={{
        height: 14, display: 'flex', alignItems: 'center', justifyContent: 'space-between',
        padding: '0 12px', flexShrink: 0,
      }}>
        <div style={{
          display: 'flex', alignItems: 'center', gap: 6,
        }}>
          {state === 'active' && <div style={{
            width: 7, height: 7, borderRadius: '50%', background: V.rec,
            boxShadow: `0 0 0 3px oklch(0.72 0.20 25 / 0.20)`,
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
        </div>
        <div style={{
          width: 36, height: 3, borderRadius: 2,
          background: 'oklch(0.50 0.010 250 / 0.5)',
        }} />
        <div style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
          <button style={overlayBtn(true)} title="Pin to top">
            <Icon d={Icons.pin} size={11} color={V.accent} />
          </button>
          <button style={overlayBtn()} title="Close overlay">
            <Icon d={Icons.close} size={11} color={V.muted} />
          </button>
        </div>
      </div>
      {/* Text stack */}
      <div style={{
        flex: 1, padding: '6px 18px 12px', display: 'flex', flexDirection: 'column',
        justifyContent: 'flex-end', gap: 2, minWidth: 0,
      }}>
        {state === 'active' && shown.map((text, i) => {
          const isLast = i === shown.length - 1;
          return (
            <div key={i} style={{
              fontSize: isLast ? 17 : 14,
              fontWeight: isLast ? 500 : 400,
              lineHeight: 1.3,
              letterSpacing: -0.15,
              color: V.text,
              opacity: opacityFor(i),
              whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis',
            }}>
              {text}
              {isLast && (
                <span style={{
                  display: 'inline-block', width: 7, height: 15, marginLeft: 3,
                  background: V.accent, verticalAlign: -2,
                  animation: 'vt-blink 0.9s steps(2) infinite',
                }} />
              )}
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
const overlayBtn = (active) => ({
  width: 22, height: 22, borderRadius: 6,
  background: active ? 'oklch(0.80 0.13 205 / 0.10)' : 'transparent',
  border: `0.5px solid ${active ? 'oklch(0.80 0.13 205 / 0.3)' : 'oklch(0.50 0.010 250 / 0.4)'}`,
  cursor: 'pointer',
  display: 'inline-flex', alignItems: 'center', justifyContent: 'center',
});

// ─── settings sheet ─────────────────────────────────────────────────────────

function SettingsSheet() {
  const [theme, setTheme] = React.useState('dark');
  const [mine, setMine] = React.useState('a');
  return (
    <div style={{
      width: 560, height: 680, borderRadius: 14,
      background: V.bg, color: V.text, fontFamily: V.font,
      border: `0.5px solid ${V.border}`,
      boxShadow: '0 30px 60px -20px rgba(0,0,0,0.6)',
      display: 'flex', flexDirection: 'column', overflow: 'hidden',
    }}>
      {/* Header */}
      <div style={{
        height: 44, padding: '0 16px',
        display: 'flex', alignItems: 'center', gap: 10,
        borderBottom: `0.5px solid ${V.border}`,
      }}>
        <TrafficLights />
        <div style={{ flex: 1, textAlign: 'center', fontSize: 13, fontWeight: 600 }}>
          Settings
        </div>
        <div style={{ width: 50 }} />
      </div>
      <div style={{ flex: 1, overflow: 'auto', padding: '20px 24px 24px' }}>
        <SettingsSection title="Soniox API key" subtitle="Stored in macOS Keychain. Never re-shown after save.">
          <div style={{
            display: 'flex', alignItems: 'center', gap: 8,
            padding: '8px 10px', borderRadius: 8,
            background: V.surface, border: `0.5px solid ${V.border}`,
          }}>
            <Icon d={Icons.key} size={13} color={V.muted} />
            <span style={{
              flex: 1, fontFamily: V.mono, fontSize: 12, color: V.muted,
              letterSpacing: 1,
            }}>••••••••••••••••••••••••••••••••</span>
            <span style={{ fontSize: 11, color: V.subtle }}>
              Last updated May 4
            </span>
            <button style={btnGhostSm()}>Replace</button>
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
      border: `0.5px solid ${V.border}`,
      fontSize: 12,
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
      background: mine ? 'oklch(0.80 0.13 205 / 0.08)' : V.surface,
      border: `0.5px solid ${mine ? 'oklch(0.80 0.13 205 / 0.5)' : V.border}`,
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
const btnGhostSm = () => ({
  padding: '4px 9px', borderRadius: 5, cursor: 'pointer',
  background: V.surface3, color: V.text,
  border: `0.5px solid ${V.borderHi}`,
  fontFamily: V.font, fontSize: 11, fontWeight: 500,
});

// ─── desktop backdrop (for context) ─────────────────────────────────────────

function Desktop({ children, height = 700, style = {} }) {
  return (
    <div style={{
      width: '100%', height, position: 'relative', overflow: 'hidden',
      background: 'linear-gradient(140deg, oklch(0.30 0.04 230) 0%, oklch(0.20 0.03 260) 50%, oklch(0.16 0.02 280) 100%)',
      display: 'flex', alignItems: 'center', justifyContent: 'center',
      ...style,
    }}>
      {/* subtle noise / dots */}
      <div style={{
        position: 'absolute', inset: 0,
        backgroundImage: 'radial-gradient(rgba(255,255,255,0.06) 1px, transparent 1px)',
        backgroundSize: '20px 20px',
        opacity: 0.4,
      }} />
      {children}
    </div>
  );
}

function ZoomBackdrop() {
  return (
    <div style={{
      position: 'absolute', inset: 0,
      display: 'flex', alignItems: 'center', justifyContent: 'center',
    }}>
      <div style={{
        width: '78%', height: '70%', borderRadius: 12,
        background: 'oklch(0.18 0.005 250 / 0.85)',
        border: '0.5px solid oklch(0.30 0.008 250)',
        boxShadow: '0 20px 50px rgba(0,0,0,0.5)',
        padding: 8, display: 'grid',
        gridTemplateColumns: '1fr 1fr 1fr',
        gap: 6,
      }}>
        {Array.from({ length: 6 }).map((_, i) => (
          <div key={i} style={{
            background: `oklch(${0.25 + (i % 3) * 0.05} 0.02 ${230 + i * 10})`,
            borderRadius: 8, position: 'relative',
          }}>
            <div style={{
              position: 'absolute', bottom: 6, left: 8,
              fontFamily: V.font, fontSize: 9, color: 'rgba(255,255,255,0.7)',
              padding: '1px 5px', background: 'rgba(0,0,0,0.4)', borderRadius: 3,
            }}>{['Linh','Marcus','Aisha','Jun','Sofia','You'][i]}</div>
          </div>
        ))}
      </div>
    </div>
  );
}

Object.assign(window, {
  MainWindow, OverlayWindow, SettingsSheet, Desktop, ZoomBackdrop,
});

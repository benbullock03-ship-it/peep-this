/* Peep This — <peep-bot> pixel desktop-pet web component (no dependencies).
   <script src="peep.js"></script>
   <peep-bot state="working" color="pink" accessory="headphones" size="64"></peep-bot>
   state: idle | thinking | working | waiting | success | error | stuck
   color: yellow | pink | lavender | blue | green | any CSS colour
   accessory: none | hardhat | headphones | glasses | bow | party | chef
              crybaby | beanie | crown | shades | flowers | halo | wizard
              packed | clear | ten | spotless | architect | supporter
   energy: 1–10 (default 3, sets 8–12fps) · hover="off" disables parallax/reaction
   JS: el.state = 'waiting'  (also .color, .accessory, .energy)
   Drawn on a 32×32 grid: sizes 32, 64, 96… are pixel-perfect. */
(function () {
  if (!window.customElements || customElements.get('peep-bot')) return;
  const N = 32, CX = 15.5;
  const PALETTE = { yellow: '#F7D64A', pink: '#F7A8C8', lavender: '#C2A7EE', blue: '#8DC7F2', green: '#A5DC93' };
  const LED = { idle: '#DDF3FF', thinking: '#C7B2FF', working: '#7FE6FF', waiting: '#FFB23E', success: '#7DFFA6', error: '#FF5D5D', stuck: '#8C97AB' };
  const STATES = Object.keys(LED);
  const hex = h => [parseInt(h.slice(1, 3), 16), parseInt(h.slice(3, 5), 16), parseInt(h.slice(5, 7), 16)];
  const u = (c, a = 255) => ((a << 24) | (c[2] << 16) | (c[1] << 8) | c[0]) >>> 0;
  const mix = (a, b, t) => a.map((v, i) => Math.round(v + (b[i] - v) * t));
  const H = h => u(hex(h));
  let probe;
  function parse(c) {
    c = PALETTE[c] || c || PALETTE.yellow;
    if (/^#[0-9a-f]{6}$/i.test(c)) return hex(c);
    probe = probe || document.createElement('canvas').getContext('2d');
    probe.fillStyle = '#F7D64A'; probe.fillStyle = c;
    return hex(/^#[0-9a-f]{6}$/i.test(probe.fillStyle) ? probe.fillStyle : PALETTE.yellow);
  }
  const hash = (x, y, s) => ((((x * 73856093) ^ (y * 19349663) ^ (s * 83492791)) >>> 0) % 1000) / 1000;
  const C = {
    eye: H('#4A2C1A'), white: H('#FFFFFF'), grey: H('#8E889A'), shadow: u([0, 0, 0], 90),
    hat: H('#F2A83A'), hatS: H('#C97F1E'), hatL: H('#FFD27A'), dark: H('#3B3446'), pad: H('#FF8FB1'), frame: H('#2A2430'),
    bow: H('#E9557A'), bowS: H('#B93A5E'), stripe: H('#7CC8FF'), pom: H('#FFE066'), chef: H('#F8F5EF'), chefS: H('#CFC8BC'),
    drop: H('#7FE6FF'), amber: H('#FFB23E'), red: H('#FF5D5D'), lav: H('#C7B2FF'), zz: H('#B8C2D6'), mint: H('#7DFFA6'), gold: H('#FFE066'),
    hair: H('#FF6FAE'), hairL: H('#FFB3D4'), hairS: H('#C93E80'), ink: H('#3A2A55'), silver: H('#C9CED8'), silverL: H('#F4F6FA'),
    knit: H('#2E2A36'), knitS: H('#1E1B24'), knitL: H('#4A4458'), goldS: H('#D9A520'), leaf: H('#5DBE6A'), wiz: H('#5B4BC4'), wizS: H('#3F3296'),
    pack: H('#C8743A'), packS: H('#8F4E22'), shineS: H('#2FA8D8')
  };
  const GLYPH = {
    '?': ['111', '001', '011', '000', '010'], '!': ['11', '11', '11', '00', '11'],
    z: ['111', '010', '111'], Z: ['1111', '0010', '0100', '1111'],
    plus: ['010', '111', '010'], dot: ['1'], drop: ['1', '1'], bolt: ['01', '11', '10']
  };
  const WING = {
    down: [9.5, 23, 2.2, 3.2], out: [11, 21.5, 2.2, 3], upA: [10.5, 17, 2.2, 5], upB: [11.5, 16, 2.2, 5.5],
    chinA: [4.5, 19, 2.4, 2], chinB: [4.5, 20, 2.4, 2], typeA: [4.5, 24, 2.4, 2], typeB: [4.5, 26, 2.4, 2]
  };
  const BASE = { by: 0, sq: 0, bx: 0, hx: 0, hy: 0, wl: 'down', wr: 'down', eye: 'dot', lx: 0, ly: 0, ant: 'up', led: 1 };
  const SPARK = [[3, 6], [27, 4], [1, 16], [28, 14], [8, 1], [24, 9]];

  const POSES = {
    idle(f, fs, s) {
      const r = Math.floor(hash(Math.floor(f / 14), 7, s) * 5);
      const flap = f % 60 >= 30 && f % 60 < 34;
      return { sq: f % 20 < 10 ? 0 : 1, lx: [-1, 0, 0, 0, 1][r], eye: f % 43 < 2 ? 'blink' : 'dot',
        wl: flap ? 'out' : 'down', wr: flap ? 'out' : 'down', marks: [] };
    },
    thinking(f) {
      const c = f % 30, m = [];
      if (c > 1) m.push(['dot', 23, 10, C.lav]);
      if (c > 3) m.push(['dot', 24, 8, C.lav]);
      if (c > 5) m.push(['?', 26, 1 + (Math.floor(f / 5) % 2), C.lav]);
      return { hx: 1, lx: 1, ly: -1, wr: Math.floor(f / 3) % 2 ? 'chinA' : 'chinB', led: Math.floor(f / 5) % 2, marks: m };
    },
    working(f) {
      const a = Math.floor(f / 2) % 2, c = f % 8, y = [9, 8, 8, 9][c % 4];
      return { sq: a, ly: 1, lx: [-1, 0, 1, 0][Math.floor(f / 6) % 4], wl: a ? 'typeA' : 'typeB', wr: a ? 'typeB' : 'typeA', led: f % 2,
        marks: [c < 4 ? ['drop', 7 - c, y, C.drop] : ['drop', 24 + (c - 4), y, C.drop]] };
    },
    waiting(f) {
      const c = f % 14;
      return { by: [0, 0, -1, -3, -4, -3, -1, 0, 0, 0, 0, 0, 0, 0][c], sq: c === 0 || c === 8 ? 1 : 0, eye: 'wide',
        wr: Math.floor(f / 2) % 2 ? 'upA' : 'upB', led: Math.floor(f / 3) % 2,
        marks: [['!', 4, 1 + (Math.floor(f / 3) % 2), C.amber]] };
    },
    success(f, fs) {
      const party = fs < 24, c = fs % 8, m = [];
      SPARK.forEach((p, i) => {
        if (!party && i > 2) return;
        const ph = party ? (f + i * 3) % 6 : (f + i * 5) % 18;
        const col = [C.mint, C.white, C.gold][i % 3];
        if (ph < 2) m.push(['dot', p[0] + 1, p[1] + 1, col]);
        else if (ph < (party ? 4 : 5)) m.push(['plus', p[0], p[1], col]);
      });
      const w = Math.floor(f / 2) % 2 ? 'upA' : 'upB';
      return party
        ? { by: [0, -2, -3, -4, -3, -2, 0, 0][c], sq: c === 7 || c === 0 ? 1 : 0, eye: 'happy', wl: w, wr: w, marks: m }
        : { by: fs % 40 < 2 ? -1 : 0, sq: f % 24 < 12 ? 0 : 1, eye: 'happy', marks: m };
    },
    error(f, fs) {
      const sh = fs < 7 ? (fs % 2 ? 1 : -1) : (fs % 35 < 4 ? (fs % 2 ? 1 : -1) : 0);
      const m = [Math.floor(f / 3) % 2 ? ['bolt', 9, 2, C.red] : ['bolt', 21, 3, C.red]];
      const c = f % 16; if (c < 8) m.push(['drop', 6, 10 + Math.floor(c / 2), C.drop]);
      return { bx: sh, sq: 1, eye: 'x', ant: 'bent', led: Math.floor(f / 2) % 2, marks: m };
    },
    stuck(f) {
      const c = f % 34, m = [];
      if (c >= 3 && c < 30) m.push(['z', 23, 8, C.zz]);
      if (c >= 11 && c < 30) m.push(['z', 26, 4, C.zz]);
      if (c >= 19 && c < 30) m.push(['Z', 27, 0, C.zz]);
      return { hx: -1, hy: 1, sq: f % 30 < 15 ? 0 : 1, eye: 'half', lx: Math.floor(f / 25) % 2 ? 1 : -1, ant: 'droop', led: 0, marks: m };
    }
  };

  const peeps = new Set();
  let running = false;
  function loop() {
    if (!peeps.size) { running = false; return; }
    requestAnimationFrame(loop);
    const t = performance.now() / 1000;
    peeps.forEach(p => p._tick(t));
  }

  class PeepBot extends HTMLElement {
    static get observedAttributes() { return ['state', 'color', 'accessory', 'energy', 'size', 'hover']; }
    constructor() {
      super();
      const sh = this.attachShadow({ mode: 'open' });
      sh.innerHTML = '<style>:host{display:inline-block;width:64px;height:64px;vertical-align:middle;flex:none}canvas{width:100%;height:100%;display:block;image-rendering:pixelated;image-rendering:crisp-edges;image-rendering:pixelated}</style><canvas width="32" height="32"></canvas>';
      this._ctx = sh.querySelector('canvas').getContext('2d');
      this._img = this._ctx.createImageData(N, N);
      this._buf = new Uint32Array(this._img.data.buffer);
      this._sil = new Uint8Array(N * N); this._wm = new Uint8Array(N * N);
      this._state = 'idle'; this._seed = Math.floor(Math.random() * 97);
      this._f = -1; this._enterF = 0; this._hoverF = -99; this._px = 0; this._py = 0; this._hov = false; this._dirty = true; this._visible = true;
      this._setColor();
      const on = () => this.getAttribute('hover') !== 'off';
      this.addEventListener('pointerenter', () => { if (on()) { this._hov = true; this._hoverF = this._f; this._dirty = true; } });
      this.addEventListener('pointermove', e => {
        if (!on()) return;
        const r = this.getBoundingClientRect();
        this._px = Math.max(-1, Math.min(1, ((e.clientX - r.left) / r.width - 0.5) * 2.4));
        this._py = Math.max(-1, Math.min(1, ((e.clientY - r.top) / r.height - 0.5) * 2.4));
        this._dirty = true;
      });
      this.addEventListener('pointerleave', () => { this._hov = false; this._px = this._py = 0; this._dirty = true; });
    }
    connectedCallback() {
      this._applySize();
      this._io = new IntersectionObserver(e => { this._visible = e[e.length - 1].isIntersecting; });
      this._io.observe(this);
      peeps.add(this);
      if (!running) { running = true; requestAnimationFrame(loop); }
    }
    disconnectedCallback() { peeps.delete(this); if (this._io) this._io.disconnect(); }
    attributeChangedCallback(n, o, v) {
      if (o === v) return;
      if (n === 'state') { v = STATES.includes(v) ? v : 'idle'; if (v !== this._state) { this._state = v; this._enterF = this._f; } }
      else if (n === 'size') this._applySize();
      else if (n === 'color') this._setColor();
      this._dirty = true;
    }
    get state() { return this._state; } set state(v) { this.setAttribute('state', v); }
    get color() { return this.getAttribute('color'); } set color(v) { this.setAttribute('color', v); }
    get accessory() { return this.getAttribute('accessory'); } set accessory(v) { this.setAttribute('accessory', v); }
    get energy() { return this._energy(); } set energy(v) { this.setAttribute('energy', v); }
    _energy() { const e = parseFloat(this.getAttribute('energy')); return isFinite(e) ? Math.max(1, Math.min(10, e)) : 3; }
    _applySize() { const s = parseFloat(this.getAttribute('size')); if (isFinite(s) && s > 0) { this.style.width = s + 'px'; this.style.height = s + 'px'; } }
    _setColor() {
      const b = parse(this.getAttribute('color')), ink = [40, 24, 52];
      this._c = { base: u(b), light: u(mix(b, [255, 255, 255], 0.5)), shade: u(mix(b, [96, 60, 120], 0.3)), line: u(mix(b, ink, 0.62)),
        beak: u(mix(b, ink, 0.42)), blush: u(mix(b, [255, 90, 130], 0.42)) };
    }
    _tick(t) {
      if (!this._visible) return;
      const fps = Math.max(8, Math.min(12, 7 + this._energy()));
      const f = Math.floor(t * fps) + this._seed;
      if (f === this._f && !this._dirty) return;
      if (this._f < 0) this._enterF = f - 10;
      this._f = f; this._dirty = false;
      this._draw(f);
    }
    _draw(f) {
      const buf = this._buf, sil = this._sil, wm = this._wm, c = this._c, st = this._state;
      const fs = f - this._enterF;
      const P = Object.assign({}, BASE, POSES[st](f, fs, this._seed));
      if (fs < 2) { P.eye = 'blink'; P.sq = 1; P.by = 0; }
      const hf = f - this._hoverF;
      if (hf < 2) { P.sq = 1; P.by = 0; } else if (hf < 4) P.by -= 1;
      const px = this._hov ? this._px : 0, py = this._hov ? this._py : 0;
      if (this._hov) { P.lx = Math.round(px); P.ly = Math.round(py); }
      const bx = P.bx + Math.round(px * 0.6), by = P.by;
      const hx = bx + P.hx + Math.round(px * 1.4), hy = by + P.hy + P.sq + Math.round(py * 0.8);
      const fx = hx + Math.round(px * 0.9), fy = hy;
      const mx = Math.round(px * 2.6), my = Math.round(py * 1.6);
      buf.fill(0); sil.fill(0);
      const set = (x, y, col) => { if (x >= 0 && y >= 0 && x < N && y < N) buf[y * N + x] = col; };
      const S = (x, y) => x >= 0 && y >= 0 && x < N && y < N && sil[y * N + x];
      const ell = (m, cx, cy, rx, ry, maxY) => {
        for (let y = Math.max(0, Math.floor(cy - ry)); y <= Math.min(N - 1, maxY == null ? N : maxY, Math.ceil(cy + ry)); y++)
          for (let x = Math.max(0, Math.floor(cx - rx)); x <= Math.min(N - 1, Math.ceil(cx + rx)); x++) {
            const dx = (x - cx) / rx, dy = (y - cy) / ry;
            if (dx * dx + dy * dy <= 1) m[y * N + x] = 1;
          }
      };
      const rect = (x0, y0, x1, y1, col, ox, oy) => { for (let y = y0; y <= y1; y++) for (let x = x0; x <= x1; x++) set(x + ox, y + oy, col); };
      const glyph = (g, x, y, col) => GLYPH[g].forEach((row, j) => { for (let i = 0; i < row.length; i++) if (row[i] === '1') set(x + i, y + j, col); });

      // ground shadow
      const sw = 10 + Math.max(-4, by);
      for (let x = Math.ceil(CX - sw); x <= Math.floor(CX + sw); x++) set(x + bx, 31, C.shadow);

      // body + head silhouette
      ell(sil, CX + bx, 24 + P.sq * 0.5 + by, 10 + P.sq * 0.5, 6 - P.sq * 0.5, 29 + by);
      ell(sil, CX + hx, 14.5 + hy, 6, 6);
      const tw = Math.floor(f / 3);
      for (let y = 0; y < N; y++) for (let x = 0; x < N; x++) {
        const i = y * N + x;
        if (sil[i]) {
          let col = c.base;
          if (!S(x + 1, y) || !S(x, y + 1) || !S(x + 1, y + 1) || !S(x, y + 2)) col = c.shade;
          else if (!S(x - 1, y) || !S(x, y - 1)) col = c.light;
          else {
            const g = hash(x - bx, y - by, 3);
            if (g < 0.13) col = c.light; else if (g > 0.95) col = c.shade;
            if (hash(x - bx, y - by, tw) < 0.012) col = C.white;
          }
          buf[i] = col;
        } else if (S(x + 1, y) || S(x - 1, y) || S(x, y + 1) || S(x, y - 1)) buf[i] = c.line;
      }

      // face
      set(10 + fx, 17 + fy, c.blush); set(11 + fx, 17 + fy, c.blush); set(20 + fx, 17 + fy, c.blush); set(21 + fx, 17 + fy, c.blush);
      set(15 + fx, 17 + fy, c.beak); set(16 + fx, 17 + fy, c.beak);
      for (const s of [-1, 1]) {
        const ex = (s < 0 ? 12 : 18) + fx + P.lx, ey = 14 + fy + P.ly, x0 = ex - 1;
        if (P.eye === 'dot') rect(ex, ey, ex + 1, ey + 1, C.eye, 0, 0);
        else if (P.eye === 'blink') rect(ex, ey + 1, ex + 1, ey + 1, C.eye, 0, 0);
        else if (P.eye === 'wide') { rect(ex, ey - 1, ex + 1, ey + 1, C.eye, 0, 0); set(ex, ey - 1, C.white); }
        else if (P.eye === 'happy') { set(x0, ey + 1, C.eye); set(x0 + 1, ey, C.eye); set(x0 + 2, ey, C.eye); set(x0 + 3, ey + 1, C.eye); }
        else if (P.eye === 'half') { rect(ex, ey, ex + 1, ey, c.shade, 0, 0); rect(ex, ey + 1, ex + 1, ey + 1, C.eye, 0, 0); }
        else if (P.eye === 'x') { const xx = s < 0 ? ex - 1 : ex; set(xx, ey - 1, C.eye); set(xx + 2, ey - 1, C.eye); set(xx + 1, ey, C.eye); set(xx, ey + 1, C.eye); set(xx + 2, ey + 1, C.eye); }
      }

      // wings
      for (const s of [-1, 1]) {
        const w = WING[s < 0 ? P.wl : P.wr];
        wm.fill(0);
        ell(wm, CX + s * w[0] + bx, w[1] + by + (w[1] > 20 ? P.sq * 0.5 : 0), w[2], w[3]);
        const Wm = (x, y) => x >= 0 && y >= 0 && x < N && y < N && wm[y * N + x];
        for (let y = 0; y < N; y++) for (let x = 0; x < N; x++) {
          const i = y * N + x;
          if (wm[i]) buf[i] = !Wm(x, y - 1) ? c.light : (!Wm(x, y + 1) ? c.shade : c.base);
          else if (Wm(x + 1, y) || Wm(x - 1, y) || Wm(x, y + 1) || Wm(x, y - 1)) buf[i] = sil[i] ? c.shade : (buf[i] === 0 || buf[i] === C.shadow ? c.line : buf[i]);
        }
      }

      // accessory
      const acc = this.getAttribute('accessory') || 'none';
      let ax = 15, ay = 8;
      const R = (x0, y0, x1, y1, col) => rect(x0, y0, x1, y1, col, hx, hy);
      if (acc === 'hardhat') {
        R(13, 6, 18, 6, C.hat); R(11, 7, 20, 7, C.hat); R(10, 8, 21, 9, C.hat); R(8, 10, 23, 10, C.hatS); R(12, 7, 13, 7, C.hatL); R(11, 8, 11, 8, C.hatL);
        ay = 5;
      } else if (acc === 'headphones') {
        R(12, 7, 19, 7, C.dark); R(10, 8, 11, 8, C.dark); R(20, 8, 21, 8, C.dark); R(9, 9, 9, 11, C.dark); R(22, 9, 22, 11, C.dark);
        R(7, 12, 9, 16, C.dark); R(22, 12, 24, 16, C.dark); R(7, 13, 7, 15, C.pad); R(24, 13, 24, 15, C.pad);
      } else if (acc === 'glasses') {
        for (const x0 of [10, 17]) { rect(x0, 12, x0 + 4, 12, C.frame, fx, fy); rect(x0, 16, x0 + 4, 16, C.frame, fx, fy); rect(x0, 12, x0, 16, C.frame, fx, fy); rect(x0 + 4, 12, x0 + 4, 16, C.frame, fx, fy); }
        rect(15, 13, 16, 13, C.frame, fx, fy);
      } else if (acc === 'bow') {
        R(17, 7, 17, 9, C.bow); R(18, 8, 18, 8, C.bow); R(20, 8, 20, 8, C.bow); R(21, 7, 21, 9, C.bow); R(19, 8, 19, 8, C.bowS); R(17, 9, 17, 9, C.bowS); R(21, 9, 21, 9, C.bowS);
      } else if (acc === 'party') {
        [[15, 16], [15, 16], [14, 17], [14, 17], [13, 18], [13, 18], [12, 19]].forEach((r, j) => { for (let x = r[0]; x <= r[1]; x++) set(x + hx, 3 + j + hy, (x + j) % 4 < 2 ? C.stripe : C.white); });
        R(15, 1, 16, 2, C.pom); ax = 10; ay = 11;
      } else if (acc === 'chef') {
        R(12, 2, 19, 2, C.chef); R(10, 3, 21, 5, C.chef); R(11, 6, 20, 6, C.chef); R(12, 7, 19, 9, C.chef); R(12, 9, 19, 9, C.chefS); R(13, 6, 13, 6, C.chefS); R(18, 6, 18, 6, C.chefS); R(15, 5, 16, 5, C.chefS);
        ax = 8; ay = 12;
      } else if (acc === 'crybaby') {
        // emo-rap look: pink swept fringe, tiny face tats, silver chain
        R(12, 8, 19, 8, C.hair); R(10, 9, 21, 9, C.hair); R(10, 10, 17, 10, C.hair); R(10, 11, 14, 11, C.hair); R(10, 12, 12, 12, C.hair); R(10, 13, 10, 13, C.hair);
        R(13, 8, 15, 8, C.hairL); R(11, 9, 13, 9, C.hairL); R(18, 10, 21, 10, C.hairS); R(15, 11, 16, 11, C.hairS); R(13, 12, 13, 12, C.hairS);
        set(13 + fx, 17 + fy, C.ink); set(13 + fx, 18 + fy, C.ink); // tear tat
        set(20 + fx, 12 + fy, C.ink); set(19 + fx, 13 + fy, C.ink); set(21 + fx, 13 + fy, C.ink); // star tat
        for (let x = 11; x <= 20; x++) set(x + bx, (x === 11 || x === 20 ? 20 : 21) + by + P.sq, x % 2 ? C.silver : C.silverL);
        set(15 + bx, 22 + by + P.sq, C.silver); set(16 + bx, 22 + by + P.sq, C.silverL); set(15 + bx, 23 + by + P.sq, C.silverL); set(16 + bx, 23 + by + P.sq, C.silver);
        ax = 17; ay = 8;
      } else if (acc === 'beanie') {
        R(13, 4, 18, 4, C.knit); R(11, 5, 20, 5, C.knit); R(10, 6, 21, 8, C.knit); R(9, 9, 22, 10, C.knitS);
        for (let x = 10; x <= 21; x += 2) R(x, 9, x, 10, C.knitL);
        R(12, 5, 13, 6, C.knitL); ay = 4;
      } else if (acc === 'crown') {
        R(10, 8, 21, 9, C.gold); R(10, 9, 21, 9, C.goldS);
        for (const x of [10, 13, 18, 21]) R(x, 6, x, 7, C.gold);
        R(13, 5, 13, 5, C.gold); R(18, 5, 18, 5, C.gold);
        set(12 + hx, 8 + hy, C.red); set(19 + hx, 8 + hy, C.stripe); set(15 + hx, 8 + hy, C.mint); set(16 + hx, 8 + hy, C.mint);
        ay = 8;
      } else if (acc === 'shades') {
        rect(10, 13, 14, 15, C.frame, fx, fy); rect(17, 13, 21, 15, C.frame, fx, fy); rect(15, 13, 16, 13, C.frame, fx, fy);
        rect(9, 13, 9, 13, C.frame, fx, fy); rect(22, 13, 22, 13, C.frame, fx, fy);
        set(11 + fx, 13 + fy, C.white); set(18 + fx, 13 + fy, C.white); set(12 + fx, 14 + fy, C.dark); set(19 + fx, 14 + fy, C.dark);
      } else if (acc === 'flowers') {
        R(11, 9, 20, 9, C.leaf);
        [[11, C.bow], [14, C.white], [17, C.pom], [20, C.lav]].forEach(([x, col]) => { R(x - 1, 8, x + 1, 8, col); R(x, 7, x, 9, col); set(x + hx, 8 + hy, C.gold); });
        ax = 15; ay = 7;
      } else if (acc === 'halo') {
        R(12, 2, 19, 2, C.gold); R(11, 3, 11, 3, C.gold); R(20, 3, 20, 3, C.gold); R(12, 4, 19, 4, C.goldS); R(13, 3, 18, 3, 0);
        ax = 10; ay = 11;
      } else if (acc === 'wizard') {
        [[15, 16], [14, 17], [14, 17], [13, 18], [13, 18], [12, 19]].forEach((r, j) => { for (let x = r[0]; x <= r[1]; x++) set(x + hx, 2 + j + hy, C.wiz); });
        R(16, 1, 17, 1, C.wiz); R(18, 0, 18, 0, C.wiz); R(10, 8, 21, 8, C.wizS); R(9, 9, 22, 9, C.wizS);
        set(14 + hx, 5 + hy, C.gold); set(17 + hx, 3 + hy, C.gold); set(16 + hx, 6 + hy, C.white);
        ax = 10; ay = 11;
      } else if (acc === 'packed') {
        // pack + bedroll over the shoulder, strap ends on the chest; rides the body
        const oy = by + P.sq;
        rect(5, 11, 8, 12, C.stripe, bx, oy); rect(5, 11, 5, 12, C.white, bx, oy);
        rect(5, 13, 8, 18, C.pack, bx, oy); rect(5, 15, 8, 15, C.packS, bx, oy); rect(5, 18, 8, 18, C.packS, bx, oy); rect(6, 16, 7, 16, C.gold, bx, oy);
        for (let i = 0; i < 3; i++) { set(9 + i + bx, 19 + i + oy, C.packS); set(22 - i + bx, 19 + i + oy, C.packS); }
      } else if (acc === 'clear') {
        R(16, 3, 18, 3, C.pom); R(15, 4, 18, 5, C.pom); R(12, 5, 12, 5, C.pom); R(12, 6, 19, 7, C.pom); R(13, 8, 18, 8, C.goldS); R(14, 7, 16, 7, C.goldS);
        R(19, 4, 20, 4, C.hat); R(19, 5, 19, 5, C.hatS); R(17, 4, 17, 4, C.eye); R(16, 3, 16, 3, C.white);
        ax = 8; ay = 12;
      } else if (acc === 'ten') {
        const oy = by + P.sq;
        rect(14, 19, 17, 20, C.red, bx, oy); rect(15, 19, 16, 20, C.white, bx, oy);
        rect(15, 21, 16, 24, C.hatS, bx, oy); rect(14, 22, 17, 23, C.hatS, bx, oy); rect(15, 22, 16, 23, C.gold, bx, oy); set(15 + bx, 22 + oy, C.white);
      } else if (acc === 'spotless') {
        R(22, 5, 22, 9, C.shineS); R(20, 7, 24, 7, C.shineS); R(21, 6, 23, 8, C.drop); R(22, 6, 22, 8, C.white); R(21, 7, 23, 7, C.white);
      } else if (acc === 'architect') {
        R(24, 6, 25, 6, C.pad); R(23, 7, 24, 7, C.silver);
        for (let i = 0; i < 3; i++) { R(22 - i, 8 + i, 22 - i, 8 + i, C.hat); R(23 - i, 8 + i, 23 - i, 8 + i, C.hatS); }
        R(19, 11, 20, 11, C.hatL); R(18, 12, 18, 12, C.dark);
      } else if (acc === 'supporter') {
        const oy = by + P.sq;
        rect(13, 20, 14, 20, C.bow, bx, oy); rect(16, 20, 17, 20, C.bow, bx, oy); rect(13, 21, 17, 21, C.bow, bx, oy); rect(14, 22, 16, 22, C.bow, bx, oy);
        set(15 + bx, 23 + oy, C.bowS); set(16 + bx, 22 + oy, C.bowS); set(17 + bx, 21 + oy, C.bowS); set(13 + bx, 20 + oy, C.white);
      }

      // antenna
      const led = hex(LED[st]), tip = u(P.led ? led : mix(led, [30, 26, 38], 0.55));
      const A = (x, y, col) => set(ax + x + hx + Math.round(px * 0.8), ay + y + hy, col);
      if (P.ant === 'up') { A(0, -1, C.grey); A(0, -2, C.grey); A(0, -3, tip); A(1, -3, tip); A(0, -4, tip); A(1, -4, tip); if (P.led) A(0, -4, C.white); }
      else if (P.ant === 'droop') { A(0, -1, C.grey); A(1, -2, C.grey); A(2, -2, C.grey); A(3, -1, tip); A(4, -1, tip); A(3, 0, tip); A(4, 0, tip); }
      else { A(0, -1, C.grey); A(-1, -2, C.grey); A(0, -3, C.grey); A(0, -4, tip); A(1, -4, tip); A(0, -5, tip); A(1, -5, tip); }

      // cartoon marks
      for (const m of P.marks) glyph(m[0], m[1] + mx, m[2] + my, m[3]);

      this._ctx.putImageData(this._img, 0, 0);
    }
  }
  customElements.define('peep-bot', PeepBot);
})();

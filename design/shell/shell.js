/* Peep This shell — renders the open window and the minimised strip from one agent list.
   Usage: PeepShell.mount(el, { mode: 'window' | 'strip', agents, usage })
   Replace SAMPLE_AGENTS with live session data; call the returned update(agents, usage) on each change. */
(function () {
  const LABEL = { idle: 'Idle', thinking: 'Thinking', working: 'Working', waiting: 'Needs you', success: 'Done', error: 'Error', stuck: 'Stuck' };
  const ATTENTION = ['waiting', 'error', 'stuck'];

  const SAMPLE_AGENTS = [
    { name: 'datamapuk-3e', state: 'thinking', color: 'lavender', acc: 'crown', say: 'Hmm… how should the JSON-LD nest?', cwd: 'C:\\Projects\\datamapuk', model: 'Opus 5.5', tool: 'Agent S-309 JSON-LD structured data', since: '38s', ctx: 18, kind: 'interactive · cli' },
    { name: 'agentview-widget-2b', state: 'waiting', color: 'blue', acc: 'crybaby', say: 'Can I edit shell.css? I need a yes.', cwd: 'C:\\Projects\\agentview-widget', model: 'Sonnet 5.5', tool: 'Edit shell.css', since: '2m 10s', ctx: 41, kind: 'interactive · cli' },
    { name: 'pollmapuk-95', state: 'working', color: 'green', acc: 'halo', say: 'Running the test suite…', cwd: 'C:\\Projects\\pollmapuk', model: 'Opus 5.5', tool: 'Bash npm test', since: '12s', ctx: 63, kind: 'interactive · cli' },
    { name: 'docs-site-7a', state: 'error', color: 'pink', acc: 'beanie', say: 'Build failed. A module is missing.', cwd: 'C:\\Projects\\docs-site', model: 'Sonnet 5.5', tool: 'Bash npm run build', since: '4m', ctx: 77, kind: 'headless · sdk' },
    { name: 'api-tests-1c', state: 'success', color: 'yellow', acc: 'wizard', say: 'All done! 14 files changed.', cwd: 'C:\\Projects\\api-tests', model: 'Haiku 5.5', tool: 'Write summary.md', since: '9m', ctx: 52, kind: 'interactive · cli' },
    { name: 'scraper-44', state: 'stuck', color: 'blue', acc: 'shades', say: 'zzz… no reply for 6 min', cwd: 'C:\\Projects\\scraper', model: 'Sonnet 5.5', tool: 'WebFetch pricing page', since: '6m', ctx: 30, kind: 'headless · sdk' }
  ];
  const SAMPLE_USAGE = { fiveHour: 48, sevenDay: 15 };

  const esc = s => String(s).replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
  const att = a => ATTENTION.includes(a.state);
  const sorted = list => list.slice().sort((a, b) => att(b) - att(a)); // attention first, otherwise keep order
  const peep = (a, size) => `<peep-bot state="${esc(a.state)}" color="${esc(a.color)}" accessory="${esc(a.acc)}" size="${size}" aria-hidden="true"></peep-bot>`;
  const chip = a => `<span class="ps-chip"><span class="ps-dot"></span>${LABEL[a.state]}</span>`;
  const bubble = a => `<span class="ps-bubble"><span>${esc(a.say)}</span></span>`;
  const meter = (label, pct) => `<span class="ps-meter">${label}<i><b style="width:${pct}%"></b></i>${pct}%</span>`;
  const cls = (a, open) => `ps-item st-${esc(a.state)}${att(a) ? ' is-att' : ''}${open ? ' is-open' : ''}`;

  function openItem(a, i) {
    return `<div class="${cls(a, true)}">
      <button class="ps-openhead" data-i="${i}" aria-expanded="true"><span class="ps-name">${esc(a.name)}</span>${chip(a)}</button>
      <div class="ps-stage">${peep(a, 96)}${bubble(a)}</div>
      <span class="ps-plank"></span>
      <div class="ps-facts">
        <div class="ps-tags"><span class="ps-tag is-st">${LABEL[a.state]} for ${esc(a.since)}</span><span class="ps-tag">${esc(a.model)}</span><span class="ps-tag">${esc(a.kind)}</span></div>
        <div class="ps-fact"><span class="ps-k">Last tool</span><span>${esc(a.tool)}</span></div>
        <div class="ps-fact"><span class="ps-k">Context</span><span class="ps-bar"><b style="width:${a.ctx}%"></b></span><span>${a.ctx}%</span></div>
        <div class="ps-path">${esc(a.cwd)}</div>
      </div>
    </div>`;
  }
  function closedItem(a, i, variant) {
    return `<div class="${cls(a)}"><button class="ps-row" data-i="${i}" aria-expanded="false">
      <span class="ps-peep">${peep(a, 48)}</span>
      <span class="ps-body">${bubble(a)}<span class="ps-meta"><span class="ps-name">${esc(a.name)}</span>${chip(a)}</span></span>
    </button><span class="ps-plank"></span></div>`;
  }
  function windowHTML(variant, agents, usage, open) {
    return `<div class="ps-head"><span class="ps-title">Peep This</span>
      <span class="ps-usage">${meter('5h', usage.fiveHour)}${meter('7d', usage.sevenDay)}</span>
      <button class="ps-icon" data-act="minimise" aria-label="Minimise to sidebar">›</button></div>
      <div class="ps-list">${agents.map((a, i) => i === open ? openItem(a, i) : closedItem(a, i, variant)).join('')}</div>`;
  }
  function stripHTML(agents) {
    return `<button class="ps-icon" data-act="expand" aria-label="Open Peep This">‹</button>` +
      agents.map((a, i) => `<button class="ps-strip-btn st-${esc(a.state)}${att(a) ? ' is-att' : ''}" data-i="${i}" aria-label="${esc(a.name)}: ${LABEL[a.state]}" title="${esc(a.name)} · ${LABEL[a.state]}">${peep(a, 40)}<span class="ps-dot"></span></button>`).join('');
  }

  function mount(el, opts) {
    opts = opts || {};
    const variant = 'pixel', mode = opts.mode || 'window';
    let agents = sorted(opts.agents || SAMPLE_AGENTS), usage = opts.usage || SAMPLE_USAGE;
    let open = opts.open == null ? -1 : opts.open;
    el.className = (mode === 'strip' ? 'ps-strip' : 'ps') + ' ps--' + variant;
    const render = () => { el.innerHTML = mode === 'strip' ? stripHTML(agents) : windowHTML(variant, agents, usage, open); };
    el.addEventListener('click', e => {
      const act = e.target.closest('[data-act]');
      if (act) { el.dispatchEvent(new CustomEvent('peepshell:' + act.dataset.act, { bubbles: true })); return; }
      const b = e.target.closest('[data-i]'); if (!b) return;
      const i = +b.dataset.i;
      if (mode === 'strip') { el.dispatchEvent(new CustomEvent('peepshell:select', { bubbles: true, detail: agents[i] })); return; }
      open = open === i ? -1 : i; render();
    });
    render();
    return { update(a, u) { agents = sorted(a); if (u) usage = u; render(); } };
  }
  window.PeepShell = { mount, SAMPLE_AGENTS, SAMPLE_USAGE };
})();

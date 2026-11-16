// Goose report renderer: one template, three kinds (map, health, topic). Reads the JSON in #report.
(() => {
  const D = JSON.parse(document.getElementById('report').textContent);
  const KINDS = { map: ['Code map', 'map.html'], health: ['Repo health', 'health.html'], topic: ['Custom topic', 'topic.html'] };
  const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
  const code = (s) => esc(s).replace(/`([^`]+)`/g, '<code>$1</code>');
  const peep = (p, size) => `<peep-bot state="${p.state}" color="${p.color}" accessory="${p.acc}" size="${size}"></peep-bot>`;
  const needs = (id, edges) => edges.filter((e) => e[0] === id).map((e) => e[1]);
  const usedBy = (id, edges) => edges.filter((e) => e[1] === id).map((e) => e[0]);

  function area(a, sessions, tag, style) {
    const here = sessions.filter((p) => p.area === a.id);
    const blocks = Array.from({ length: Math.min(16, Math.ceil(a.files / 3)) }, (_, i) => `<i${i < (a.editing || 0) ? ' class="on"' : ''}></i>`).join('');
    const label = `${a.id}, ${a.files} files` + (here.length ? `, ${here.length} session${here.length > 1 ? 's' : ''} editing` : '');
    return `<${tag} class="gnode${here.some((p) => p.state === 'waiting') ? ' att' : ''}" data-id="${esc(a.id)}"${style ? ` style="${style}"` : ''}${tag === 'button' ? ` aria-label="${esc(label)}"` : ''}>
      <div class="gstand">${here.slice(0, 2).map((p) => peep(p, 32)).join('')}</div><span class="gplank"></span>
      <div class="gcard"><div class="gn">${esc(a.id)}<span>${a.files}</span></div><div class="gfiles">${blocks}</div></div></${tag}>`;
  }

  // Demo layout: column = depth, links between neighbouring columns only, one lane per row. The build brings its own.
  function drawn(areas, edges, sessions, pitch, pad, rowPitch, tag) {
    const at = {};
    let html = areas.map((a) => { const x = pad + a.col * pitch, y = pad + a.row * rowPitch; at[a.id] = { x, y, r: a.row }; return area(a, sessions, tag, `left:${x}px;top:${y}px`); }).join('');
    edges.forEach(([a, b]) => {
      const A = at[a], B = at[b], x1 = A.x + 120, y1 = A.y + 57, y2 = B.y + 57, mx = x1 + 12 + A.r * 12;
      [[x1, y1, mx - x1 + 2, 2], [mx, Math.min(y1, y2), 2, Math.abs(y2 - y1) + 2], [mx, y2, B.x - mx, 2], [B.x - 6, y2 - 2, 6, 6]].forEach(([x, y, w, h]) => {
        html += `<span class="gedge" data-a="${esc(a)}" data-b="${esc(b)}" style="left:${x}px;top:${y}px;width:${w}px;height:${h}px"></span>`;
      });
    });
    const w = 2 * pad + Math.max(...areas.map((a) => a.col)) * pitch + 120, h = 2 * pad + Math.max(...areas.map((a) => a.row)) * rowPitch + 78;
    return { html, w, h };
  }

  function map() {
    const S = D.sessions, g = drawn(D.areas, D.edges, S, 240, 20, 112, 'button');
    const chips = (ids, none) => ids.length ? `<div class="go-chips">${ids.map((i) => `<span class="go-chip">${esc(i)}</span>`).join('')}</div>` : '';
    return `<div class="go-mapwrap">
      <div class="go-mapbox"><div class="go-map" id="map" style="width:${g.w}px;height:${g.h}px">${g.html}</div></div>
      <div class="go-list">${D.areas.map((a) => { const n = needs(a.id, D.edges), u = usedBy(a.id, D.edges);
        return `<div class="go-arow">${area(a, S, 'div')}<div class="go-needs"><span>${n.length ? 'needs' : 'needs nothing'}</span>${chips(n)}<span>${u.length ? 'used by' : 'used by nothing'}</span>${chips(u)}</div></div>`; }).join('')}</div>
      <div class="go-side"><h2 class="go-h">Sessions on the map</h2>${S.length ? S.map((p) => `<div class="go-sess st-${p.state}"><span class="who">${peep(p, 32)}<span class="go-plank"></span></span><span class="nm">${esc(p.name)}<small>on ${esc(p.area)}</small></span><span class="go-state">${p.state}</span></div>`).join('')
        : '<p class="go-none">No sessions are editing this repo right now. The map still shows what needs what.</p>'}</div></div>`;
  }

  function health() {
    const row = (f) => `<li><span class="go-path">${esc(f.path)}</span>${f.num ? `<span class="go-num">${esc(f.num)}</span>` : ''}${f.tk ? `<span class="go-tk${f.blocked ? ' blocked' : ''}">${esc(f.tk)}</span>` : ''}${f.bar ? `<span class="go-bar${f.bar >= 9 ? ' red' : f.bar >= 7 ? ' amb' : ''}"><i style="--n:${f.bar}"></i></span>` : ''}<small>${esc(f.small)}</small></li>`;
    return `<div class="go-grid">${D.cards.map((c) => `<section class="go-card"><header><h2>${esc(c.title)}</h2><span class="go-n${c.rows.length ? '' : ' ok'}">${c.rows.length || 'clear'}</span></header>${c.rows.length ? `<ul class="go-rows">${c.rows.map(row).join('')}</ul>` : `<p class="go-clearline"><i class="go-ok"></i>${esc(c.clear)}</p>`}</section>`).join('')}</div>
      <h2 class="go-h">Patterns</h2><div class="go-grid">${D.patterns.map((p) => `<article class="go-pat"><h3>${esc(p.lesson)}</h3><p>${esc(p.why)}</p><details><summary>Explain like I'm 5</summary><p>${esc(p.eli5)}</p></details><span class="src">Source: <a href="${esc(p.href)}">${esc(p.source)}</a></span></article>`).join('')}</div>`;
  }

  function topic() {
    const g = drawn(D.slice.areas, D.slice.edges, D.sessions || [], 164, 12, 104, 'div');
    return `<div><h1 class="go-q">${esc(D.question)}</h1><p class="go-asked">${esc(D.asked)}</p></div>
      <div class="go-topic"><article class="go-ans"><div class="go-short"><h2 class="go-h">In short</h2><p>${code(D.short)}</p></div>${D.answer.map((b) => b.h ? `<h2>${esc(b.h)}</h2>` : `<p>${code(b.p)}</p>`).join('')}</article>
      <aside class="go-aside"><h2 class="go-h">Files involved</h2><ul class="go-rows" style="border:2px solid var(--ps-edge)">${D.files.map((f) => `<li><span class="go-path">${esc(f.path)}</span><small>${esc(f.why)}</small></li>`).join('')}</ul>
      <h2 class="go-h">On the map</h2><div class="go-slice" style="height:${g.h}px">${g.html}</div><p class="go-none">${esc(D.slice.note)} <a href="map.html">Open the whole map</a></p></aside></div>`;
  }

  const body = { map, health, topic }[D.kind]();
  const legend = D.kind === 'map' ? '<span class="wide"><i class="ln"></i>needs, ends at the area needed</span><span><i class="sq"></i>3 files</span><span><i class="sq on"></i>being edited</span>' : '';
  document.getElementById('go').innerHTML = `<div class="go-win">
    <header class="go-head"><span class="go-title"><span class="fl"></span>Goose · ${KINDS[D.kind][0]}</span><span class="go-sub"><b>${esc(D.repo)}</b></span><span class="go-sub">${esc(D.date)}</span><span class="go-sub">${esc(D.commit)}</span></header>
    <nav class="go-tabs" aria-label="Reports">${Object.entries(KINDS).map(([k, [name, href]]) => `<a href="${href}"${k === D.kind ? ' aria-current="page"' : ''}>${name}</a>`).join('')}</nav>
    <main class="go-main">${D.lead ? `<div class="go-lead"><span class="go-perch">${peep({ state: D.peep || 'idle', color: 'yellow', acc: 'none' }, 48)}<span class="go-plank"></span></span><div class="go-bub">${esc(D.lead)}</div></div>` : ''}${body}</main>
    <footer class="go-foot">${legend}${D.foot.map((t) => `<span>${esc(t)}</span>`).join('')}<span>made by /peep:goose</span></footer></div>`;

  // Hover or focus an area: its links and neighbours stay lit, the rest dim.
  const m = document.getElementById('map');
  if (m) {
    const mark = (id) => {
      const near = id ? new Set([id, ...needs(id, D.edges), ...usedBy(id, D.edges)]) : null;
      m.querySelectorAll('.gnode').forEach((n) => { n.classList.toggle('dim', !!near && !near.has(n.dataset.id)); n.classList.toggle('hot', n.dataset.id === id); });
      m.querySelectorAll('.gedge').forEach((s) => { const on = s.dataset.a === id || s.dataset.b === id; s.classList.toggle('hot', on); s.classList.toggle('dim', !!id && !on); });
    };
    m.querySelectorAll('.gnode').forEach((n) => { n.onmouseenter = n.onfocus = () => mark(n.dataset.id); n.onmouseleave = n.onblur = () => mark(null); });
  }
})();

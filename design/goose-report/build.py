"""Goose report: one template, three self-contained pages (map.html, health.html, topic.html). No network at view time.
Inlines the tokens (light theme moved under prefers-color-scheme), goose.css, the bundled fonts, peep.js and the floppy sheets."""
import base64, json, os, re

HERE = os.path.dirname(os.path.abspath(__file__))
DESIGN = os.path.join(HERE, '..')
FONTS = os.path.join(DESIGN, '..', 'src', 'assets', 'fonts')

def read(*p):
    with open(os.path.join(*p), encoding='utf-8') as f: return f.read()
def b64(*p):
    with open(os.path.join(*p), 'rb') as f: return base64.b64encode(f.read()).decode()

def tokens(path):
    css = re.sub(r'/\*.*?\*/', '', read(path), flags=re.S)
    dark = re.search(r':root\s*\{(.*?)\n\}', css, re.S)
    light = re.search(r':root\[data-theme="light"\]\s*\{(.*?)\n\}', css, re.S)
    out = ':root {' + dark.group(1) + '\n}\n'
    if light: out += '@media (prefers-color-scheme: light) {\n:root {' + light.group(1) + '\n}\n}\n'
    return out

FACES = [('Fredoka', '600', 'fredoka-latin-600-normal'), ('IBM Plex Sans', '400', 'ibm-plex-sans-latin-400-normal'), ('IBM Plex Sans', '600', 'ibm-plex-sans-latin-600-normal'),
         ('IBM Plex Mono', '400', 'ibm-plex-mono-latin-400-normal'), ('IBM Plex Mono', '500 600', 'ibm-plex-mono-latin-500-normal')]  # Mono 600 is not bundled
PARTS = {
    'FONTS': '\n'.join(f"@font-face {{ font-family: '{fam}'; font-weight: {w}; src: url(data:font/woff2;base64,{b64(FONTS, f + '.woff2')}) format('woff2'); }}" for fam, w, f in FACES),
    'TOKENS': ''.join(tokens(os.path.join(DESIGN, d, 'tokens.css')) for d in ('skill-pack', 'skill-pack-2', 'goose-report')),
    'CSS': read(HERE, 'goose.css'),
    'PEEP': read(DESIGN, 'peep.js').replace('</script', '<\\/script'),
    'JS': read(HERE, 'report.js'),
    'FLOPPY_DARK': 'data:image/png;base64,' + b64(DESIGN, 'skill-pack-2', 'sprites', 'floppies-dark.png'),
    'FLOPPY_LIGHT': 'data:image/png;base64,' + b64(DESIGN, 'skill-pack-2', 'sprites', 'floppies-light.png'),
}

# ---------------- dummy data ----------------
META = {'repo': 'agentview-widget', 'date': '2026-11-16', 'commit': '4f5e9c4'}
def A(id, col, row, files, editing=0): return {'id': id, 'col': col, 'row': row, 'files': files, 'editing': editing}
def S(name, state, color, acc, area): return {'name': name, 'state': state, 'color': color, 'acc': acc, 'area': area}
def F(path, num, small, bar=0, tk='', blocked=False): return {'path': path, 'num': num, 'small': small, 'bar': bar, 'tk': tk, 'blocked': blocked}

MAP = {**META, 'kind': 'map',
    'lead': '12 areas, 14 links. Four sessions are editing three areas; two of them share source, and one is waiting.',
    'areas': [A('ui', 0, 0, 34, 2), A('settings', 0, 1, 6), A('plugin', 0, 2, 18), A('bridge', 1, 0, 7), A('look', 1, 1, 5), A('skills', 1, 2, 14),
              A('commands', 2, 0, 9), A('state', 2, 1, 6, 1), A('installer', 2, 2, 8), A('source', 3, 0, 11, 3), A('server', 3, 1, 5), A('store', 3, 2, 4)],
    'edges': [['ui', 'bridge'], ['ui', 'look'], ['settings', 'bridge'], ['settings', 'look'], ['plugin', 'skills'], ['bridge', 'commands'], ['bridge', 'state'],
              ['look', 'commands'], ['skills', 'installer'], ['commands', 'source'], ['commands', 'store'], ['state', 'source'], ['state', 'server'], ['installer', 'store']],
    'sessions': [S('peep-this-7a', 'working', 'yellow', 'headphones', 'ui'), S('peep-this-7b', 'thinking', 'lavender', 'glasses', 'state'),
                 S('peep-this-7c', 'working', 'blue', 'hardhat', 'source'), S('peep-this-7d', 'waiting', 'pink', 'bow', 'source')],
    'foot': []}

HEALTH = {**META, 'kind': 'health', 'peep': 'thinking',
    'lead': '11 findings in 4 groups, none urgent. Start with src/styles.css: it is the busiest file and the biggest.',
    'cards': [
        {'title': 'Hot spots', 'clear': 'No file changed more than 10 times in 30 days.', 'rows': [
            F('src/styles.css', '41 changes', 'touched by 6 tickets in 30 days', 10), F('src/rowView.ts', '33 changes', 'touched by 5 tickets', 8),
            F('src-tauri/src/state.rs', '27 changes', 'touched by 4 tickets', 7), F('src/rows.ts', '22 changes', 'touched by 4 tickets', 5)]},
        {'title': 'Biggest files', 'clear': 'No file is over 800 lines.', 'rows': [
            F('src/styles.css', '1,912 lines', 'over the 800-line mark', 10), F('src-tauri/src/state.rs', '1,104 lines', 'over the 800-line mark', 6),
            F('src/rowView.ts', '866 lines', 'just over the mark', 5)]},
        {'title': 'Untested areas', 'clear': 'Every area has at least one test.', 'rows': [
            F('skills', '0 of 14', 'no test imports anything in plugin/skills/'), F('flyout', '0 of 3', 'no test imports src/flyoutPage.ts')]},
        {'title': 'Stale tickets', 'clear': 'No ticket has sat in doing or blocked for over 7 days.', 'rows': [
            F('AV-93 Flyout approval card', '', 'no commit mentions it since 2026-11-04', tk='doing · 12 days'),
            F('AV-88 Rules store', '', 'blocked on AV-54, which closed 9 days ago', tk='blocked · 9 days', blocked=True)]},
    ],
    'patterns': [
        {'lesson': 'A file that is both the busiest and the biggest is where sessions collide.',
         'why': 'src/styles.css leads both lists. Splitting it by surface (rows, strip, flyout) gives each ticket its own file to change.',
         'eli5': 'One toy box that everybody digs in at once. A few smaller boxes, and nobody bumps heads.',
         'source': 'A Philosophy of Software Design, ch. 4', 'href': 'https://web.stanford.edu/~ouster/cgi-bin/book.php'},
        {'lesson': 'Code with no test cannot tell you when it breaks.',
         'why': 'Two areas have no test at all. One test that imports the area is enough to make Coderot useful there.',
         'eli5': 'A smoke alarm with no battery looks fine right up until there is a fire.',
         'source': 'Test Coverage, Martin Fowler', 'href': 'https://martinfowler.com/bliki/TestCoverage.html'},
        {'lesson': 'A ticket left in doing hides who is working on what.',
         'why': 'AV-93 has been doing for 12 days with no commit. Park it with a reason, or pick it back up.',
         'eli5': 'A "back in 5 minutes" sign that has been on the door all week.',
         'source': 'CONTEXT.md, ticket statuses', 'href': 'CONTEXT.md'},
    ],
    'foot': ['hot spots and stale tickets look back 30 days']}

TOPIC = {**META, 'kind': 'topic',
    'question': 'What happens between a hook firing and a peep changing state?',
    'asked': 'asked by the owner · answered from 8 files in 4 areas',
    'short': 'Claude Code posts the hook to a loopback port. The server matches it to a session by `sessionId`, one pure function works out the new state, and the whole list is pushed to the window, which only redraws.',
    'answer': [
        {'h': '1 · The hook posts'},
        {'p': "The integration install adds one entry per hook to `~/.claude/settings.json`. Each entry posts the hook's JSON to the widget's port on 127.0.0.1. Nothing is read back, so a slow widget can never hold Claude Code up."},
        {'h': '2 · The server finds the session'},
        {'p': '`server.rs` accepts the post and hands it to the Claude source, which parses the payload and pulls out `sessionId`. If no session file has been seen for that id yet, the event is held until the sessions directory catches up.'},
        {'h': '3 · One function decides'},
        {'p': "`state.rs` takes the session's current state and the event and returns the next state. `PreToolUse` gives working, the matching `PostToolUse` gives thinking, `Notification` gives waiting, `Stop` gives success. It reads no clock and no files, which is why every transition has a test."},
        {'h': '4 · The snapshot goes out'},
        {'p': "The backend emits the full session list as one `sessions` event. The frontend does not patch anything: `rows.ts` sorts attention states first and `rowView.ts` sets `el.state` on the row's peep, which is the only line that touches the art."},
        {'h': '5 · Where it can go wrong'},
        {'p': 'Three places. A session started before the hooks were installed sends nothing, so it falls back to the transcript tail and lags by a second or two. A busy session with no event for 6 minutes is marked stuck by a timer, not by a hook. And a hook that arrives for a dead PID is dropped.'},
        {'h': 'Read first'},
        {'p': '`state.rs`, then its tests. The rest is plumbing around that one function.'},
    ],
    'files': [{'path': p, 'why': w} for p, w in [
        ('src-tauri/src/state.rs', 'the pure state function'), ('src-tauri/src/server.rs', 'loopback port, accepts hook posts'),
        ('src-tauri/src/source/claude/hooks.rs', 'parses hook payloads'), ('src-tauri/src/source/claude/sessions.rs', 'watches ~/.claude/sessions'),
        ('src-tauri/src/snapshot.rs', 'builds and emits the sessions event'), ('src/rows.ts', 'sort order'),
        ('src/rowView.ts', "sets the peep's state"), ('src/vendor/peep.js', 'the creature; not changed')]],
    'slice': {'areas': [A('ui', 0, 0, 34), A('server', 0, 1, 5), A('state', 1, 0, 6, 1), A('source', 1, 1, 11)],
              'edges': [['ui', 'state'], ['server', 'state'], ['server', 'source']], 'note': '4 of 12 areas.'},
    'sessions': [S('peep-this-7b', 'thinking', 'lavender', 'glasses', 'state')],
    'foot': ['answered from the code at 4f5e9c4; it may be stale after that']}

TITLES = {'map': 'Code map', 'health': 'Repo health', 'topic': 'Custom topic'}
template = read(HERE, 'template.html')
for data in (MAP, HEALTH, TOPIC):
    page = template.replace('{{TITLE}}', f"Goose · {TITLES[data['kind']]} · {data['repo']}")
    page = page.replace('{{DATA}}', json.dumps(data, ensure_ascii=False).replace('</', '<\\/'))
    for k, v in PARTS.items(): page = page.replace('{{' + k + '}}', v)
    out = os.path.join(HERE, data['kind'] + '.html')
    with open(out, 'w', encoding='utf-8', newline='\n') as f: f.write(page)
    print(data['kind'] + '.html', os.path.getsize(out), 'bytes')

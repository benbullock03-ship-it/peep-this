import type { Identity } from "./identity";
import { cwdKey } from "./look";
import type { Session } from "./model";
import type { IdentityOf } from "./rowView";

const ICON_PX = 96;
const FRAME_TRIES = 40;

/** One `waiting` frame of the project's peep as PNG bytes, scaled up without smoothing; null if it never drew. */
async function drawIcon(id: Identity): Promise<number[] | null> {
  const peep = document.createElement("peep-bot");
  for (const [k, v] of [["state", "waiting"], ["color", id.color], ["accessory", id.accessory], ["size", "32"], ["hover", "off"]]) peep.setAttribute(k, v);
  // In view but unseen: the element only draws while its IntersectionObserver says it is on screen.
  peep.style.cssText = "position:fixed;left:0;top:0;opacity:0;pointer-events:none";
  peep.setAttribute("aria-hidden", "true");
  document.body.appendChild(peep);
  try {
    const src = peep.shadowRoot?.querySelector("canvas");
    if (!src) return null;
    const ctx = src.getContext("2d")!;
    for (let i = 0; i < FRAME_TRIES; i++) {
      await new Promise((r) => setTimeout(r, 50));
      if (ctx.getImageData(0, 0, src.width, src.height).data.some((v, j) => j % 4 === 3 && v > 0)) break;
      if (i === FRAME_TRIES - 1) return null;
    }
    const out = document.createElement("canvas");
    out.width = out.height = ICON_PX;
    const o = out.getContext("2d")!;
    o.imageSmoothingEnabled = false;
    o.drawImage(src, 0, 0, ICON_PX, ICON_PX);
    const blob = await new Promise<Blob | null>((r) => out.toBlob(r, "image/png"));
    return blob ? Array.from(new Uint8Array(await blob.arrayBuffer())) : null;
  } finally {
    peep.remove();
  }
}

/** AV-113: keeps each live project's toast icon (`set_toast_icon`) matching the look it wears. */
export function createToastIcons(send: (cwd: string, png: number[]) => Promise<unknown>) {
  const sent = new Map<string, string>();
  let busy = false;
  let pending: { sessions: readonly Session[]; identityOf: IdentityOf } | null = null;

  async function run() {
    busy = true;
    while (pending) {
      const { sessions, identityOf } = pending;
      pending = null;
      for (const s of sessions) {
        const key = cwdKey(s.cwd);
        const id = identityOf(s);
        const look = `${id.color}/${id.accessory}`;
        if (sent.get(key) === look) continue;
        const png = await drawIcon(id);
        if (!png) continue;
        try {
          await send(s.cwd, png);
          sent.set(key, look);
        } catch (err) {
          console.error("peepthis: set_toast_icon failed", err);
        }
      }
    }
    busy = false;
  }

  return (sessions: readonly Session[], identityOf: IdentityOf) => {
    pending = { sessions: sessions.filter((s) => s.alive), identityOf };
    if (!busy) void run();
  };
}

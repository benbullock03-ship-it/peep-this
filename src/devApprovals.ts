// Dev-only stand-in for the approval card (AV-23): lists held requests with Allow/Deny.
import type { Session } from "./model";

type Decide = (requestId: string, decision: "allow" | "deny", reason?: string) => Promise<boolean>;

function summary(input: Record<string, unknown>): string {
  const v = input.command ?? input.file_path ?? input.notebook_path ?? input.url ?? "";
  return String(v).split("\n")[0].slice(0, 80);
}

export function createDevApprovals(host: HTMLElement, decide: Decide) {
  const box = document.createElement("div");
  box.className = "dev-approvals";
  box.style.cssText =
    "font:11px 'IBM Plex Mono',monospace;color:#EDE9F2;background:#1A1820;border-top:2px solid #F7D64A;padding:4px 8px;";
  host.appendChild(box);
  return (sessions: Session[]) => {
    const items = sessions.flatMap((s) => (s.pending ?? []).map((p) => ({ s, p })));
    box.hidden = items.length === 0;
    box.replaceChildren(
      ...items.map(({ s, p }) => {
        const row = document.createElement("div");
        const left = Math.max(0, Math.round((p.releaseAt - Date.now()) / 1000));
        row.textContent = `[dev] ${s.name} ${p.tool} ${summary(p.input)} (${p.risk.tier}, ${left}s) `;
        for (const [label, decision] of [["Allow", "allow"], ["Deny", "deny"]] as const) {
          const b = document.createElement("button");
          b.textContent = label;
          b.onclick = () => decide(p.id, decision, decision === "deny" ? "Denied from the dev stub" : undefined);
          row.appendChild(b);
        }
        return row;
      }),
    );
  };
}

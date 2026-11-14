import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  byOldest,
  byteSpanToChars,
  createHold,
  diffFor,
  DIFF_MAX_LINES,
  displayPath,
  formatArgs,
  lineDiff,
  pushReason,
  riskParts,
} from "./approval";
import type { PendingApproval } from "./model";

const show = (lines: ReturnType<typeof lineDiff>) => lines.map((l) => `${{ same: " ", add: "+", del: "-" }[l.kind]}${l.text}`);

describe("lineDiff", () => {
  it("keeps shared lines and marks the change", () => {
    expect(show(lineDiff("a\nb\nc", "a\nB\nc"))).toEqual([" a", "-b", "+B", " c"]);
  });
  it("finds insertions in the middle", () => {
    expect(show(lineDiff("a\nc", "a\nb\nc"))).toEqual([" a", "+b", " c"]);
  });
  it("treats CRLF as LF", () => {
    expect(show(lineDiff("a\r\nb", "a\nb"))).toEqual([" a", " b"]);
  });
  it("shows a new file as all additions and empty as nothing", () => {
    expect(show(lineDiff("", "x\ny"))).toEqual(["+x", "+y"]);
    expect(lineDiff("", "")).toEqual([]);
  });
  it("falls back to delete-then-add for huge changes", () => {
    const a = Array.from({ length: 600 }, (_, i) => `a${i}`).join("\n");
    const b = Array.from({ length: 600 }, (_, i) => `b${i}`).join("\n");
    const d = lineDiff(a, b);
    expect(d.slice(0, 600).every((l) => l.kind === "del")).toBe(true);
    expect(d.slice(600).every((l) => l.kind === "add")).toBe(true);
  });
});

describe("diffFor", () => {
  it("diffs Edit, Write and MultiEdit; not Bash", () => {
    expect(show(diffFor("Edit", { old_string: "x", new_string: "y" })!.lines)).toEqual(["-x", "+y"]);
    expect(show(diffFor("Write", { content: "hi" })!.lines)).toEqual(["+hi"]);
    const multi = diffFor("MultiEdit", {
      edits: [
        { old_string: "a", new_string: "b" },
        { old_string: "c", new_string: "d" },
      ],
    })!;
    expect(show(multi.lines)).toEqual(["-a", "+b", " …", "-c", "+d"]);
    expect(diffFor("Bash", { command: "ls" })).toBeNull();
  });
  it("caps long diffs and counts the rest", () => {
    const d = diffFor("Write", { content: Array.from({ length: DIFF_MAX_LINES + 25 }, () => "l").join("\n") })!;
    expect(d.lines).toHaveLength(DIFF_MAX_LINES);
    expect(d.hidden).toBe(25);
  });
});

describe("formatArgs", () => {
  it("drops empties, stringifies objects, truncates and caps", () => {
    const out = formatArgs({ a: "x", b: "", c: null, d: { k: 1 }, e: "y".repeat(400), f: 1, g: 2, h: 3, i: 4 });
    expect(out.map((a) => a.key)).toEqual(["a", "d", "e", "f", "g", "h"]);
    expect(out[1].value).toBe('{"k":1}');
    expect(out[2].value).toHaveLength(160);
    expect(out[2].value.endsWith("…")).toBe(true);
  });
});

describe("byteSpanToChars", () => {
  it("is identity for ASCII", () => {
    expect(byteSpanToChars("rm -rf x", [0, 6])).toEqual([0, 6]);
  });
  it("maps past multi-byte characters", () => {
    const s = "echo é && rm -rf /";
    const at = new TextEncoder().encode("echo é && ").length;
    const r = byteSpanToChars(s, [at, at + 6])!;
    expect(s.slice(...r)).toBe("rm -rf");
  });
  it("rejects spans that split a character or run past the end", () => {
    expect(byteSpanToChars("é", [1, 2])).toBeNull();
    expect(byteSpanToChars("ab", [0, 9])).toBeNull();
  });
  it("accepts a span ending at the string end", () => {
    expect(byteSpanToChars("git push", [4, 8])).toEqual([4, 8]);
  });
});

const pending = (over: Partial<PendingApproval>): PendingApproval => ({
  id: "r1",
  tool: "Bash",
  input: { command: "git reset --hard" },
  cwd: "C:/p",
  receivedAt: 0,
  releaseAt: 60_000,
  risk: { tier: "destructive", field: "command", span: [4, 16], reason: "" },
  ...over,
});

describe("riskParts", () => {
  it("splits around the span on the matching field only", () => {
    expect(riskParts(pending({}), "command")).toEqual({ before: "git ", risky: "reset --hard", after: "" });
    expect(riskParts(pending({}), "file_path")).toBeNull();
    expect(riskParts(pending({ risk: { tier: "shell", reason: "" } }), "command")).toBeNull();
  });
});

describe("displayPath and ordering", () => {
  it("shortens paths inside cwd across separators and case", () => {
    expect(displayPath("C:\\Proj\\src\\a.ts", "c:/proj")).toBe("src/a.ts");
    expect(displayPath("D:/other/a.ts", "C:/proj")).toBe("D:/other/a.ts");
    expect(displayPath("C:/projx/a.ts", "C:/proj")).toBe("C:/projx/a.ts");
  });
  it("sorts oldest first", () => {
    const list = [pending({ id: "b", receivedAt: 5 }), pending({ id: "a", receivedAt: 1 })];
    expect(list.sort(byOldest).map((p) => p.id)).toEqual(["a", "b"]);
  });
});

describe("pushReason", () => {
  it("keeps the last three, newest first, without duplicates", () => {
    expect(pushReason(["a", "b", "c"], "d")).toEqual(["d", "a", "b"]);
    expect(pushReason(["a", "b", "c"], " b ")).toEqual(["b", "a", "c"]);
    expect(pushReason(["a"], "  ")).toEqual(["a"]);
  });
});

describe("createHold", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it("fires only after the full hold", () => {
    const done = vi.fn();
    const h = createHold(done, 1000);
    h.start();
    vi.advanceTimersByTime(999);
    expect(done).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(done).toHaveBeenCalledTimes(1);
    expect(h.active).toBe(false);
  });
  it("does nothing when released early, and restarts from zero", () => {
    const done = vi.fn();
    const h = createHold(done, 1000);
    h.start();
    vi.advanceTimersByTime(900);
    h.cancel();
    vi.advanceTimersByTime(500);
    expect(done).not.toHaveBeenCalled();
    h.start();
    vi.advanceTimersByTime(900);
    expect(done).not.toHaveBeenCalled();
    vi.advanceTimersByTime(100);
    expect(done).toHaveBeenCalledTimes(1);
  });
  it("ignores a second start while held", () => {
    const done = vi.fn();
    const h = createHold(done, 1000);
    h.start();
    vi.advanceTimersByTime(600);
    h.start();
    vi.advanceTimersByTime(400);
    expect(done).toHaveBeenCalledTimes(1);
  });
});

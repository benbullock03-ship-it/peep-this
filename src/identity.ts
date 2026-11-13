export const COLORS = ["yellow", "pink", "lavender", "blue", "green"] as const;
export const ACCESSORIES = ["none", "hardhat", "headphones", "glasses", "bow", "party", "chef"] as const;

export type Color = (typeof COLORS)[number];
export type Accessory = (typeof ACCESSORIES)[number];
export type Identity = { color: Color; accessory: Accessory };

export function normaliseCwd(cwd: string): string {
  return cwd.trim().toLowerCase().replace(/\\/g, "/").replace(/\/+$/, "");
}

// 32-bit FNV-1a over UTF-8 bytes.
export function fnv1a(input: string): number {
  let h = 0x811c9dc5;
  for (const b of new TextEncoder().encode(input)) {
    h ^= b;
    h = Math.imul(h, 0x01000193);
  }
  return h >>> 0;
}

// Colour from the low bits, accessory from the high half, so the two vary independently.
export function identityFor(cwd: string): Identity {
  const h = fnv1a(normaliseCwd(cwd));
  return { color: COLORS[h % COLORS.length], accessory: ACCESSORIES[(h >>> 16) % ACCESSORIES.length] };
}

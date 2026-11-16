import { COLORS, fnv1a, identityFor, normaliseCwd, pickerAccessories, type Color, type Identity } from "./identity";
import type { Session } from "./model";
import type { IdentityOf } from "./rowView";

/** AV-113: a user's pick for one project; a missing field keeps the hashed one. Mirrors `look.rs`. */
export type Look = { color?: string; accessory?: string };
export type Looks = Readonly<Record<string, Look>>;

/** The settings key for a project: its cwd hash as 8 hex digits, the same in `look.rs` `cwd_key`. */
export function cwdKey(cwd: string): string {
  return fnv1a(normaliseCwd(cwd)).toString(16).padStart(8, "0");
}

/** The look worn: each valid override field over the hashed identity. */
export function resolveLook(base: Identity, look: Look | undefined, accessories: readonly string[] = pickerAccessories()): Identity {
  const color = look?.color && (COLORS as readonly string[]).includes(look.color) ? (look.color as Color) : base.color;
  const accessory = look?.accessory && accessories.includes(look.accessory) ? look.accessory : base.accessory;
  return { color, accessory };
}

/** The look after picking `field` = `value`; null when nothing is left to override. */
export function pick(base: Identity, current: Look | undefined, field: keyof Look, value: string): Look | null {
  const next: Look = { ...current, [field]: value };
  if (next.color === base.color) delete next.color;
  if (next.accessory === base.accessory) delete next.accessory;
  return next.color || next.accessory ? next : null;
}

export type LookStore = {
  identityOf: IdentityOf;
  /** The hashed identity, before any override. */
  baseOf: IdentityOf;
  lookOf: (s: Session) => Look | undefined;
  /** For a project known only by its folder (the closed shelf): the hashed look plus any override. */
  forCwd: (cwd: string) => Identity;
  accessories: () => readonly string[];
  set: (s: Session, look: Look | null) => void;
  load: (looks: Looks) => void;
};

/** Holds the overrides; `save` persists one change and `changed` re-renders every surface. */
export function createLookStore(
  baseOf: IdentityOf,
  save: (key: string, look: Look | null) => Promise<Looks>,
  changed: () => void,
): LookStore {
  let looks: Looks = {};
  const lookOf = (s: Session) => looks[cwdKey(s.cwd)];
  const load = (next: Looks) => {
    looks = next;
    changed();
  };
  return {
    identityOf: (s) => resolveLook(baseOf(s), lookOf(s)),
    baseOf,
    lookOf,
    forCwd: (cwd) => resolveLook(identityFor(cwd), looks[cwdKey(cwd)]),
    accessories: () => pickerAccessories(),
    load,
    set(s, look) {
      const key = cwdKey(s.cwd);
      const prev = looks;
      const next = { ...looks };
      if (look) next[key] = look;
      else delete next[key];
      load(next);
      save(key, look).then(load, (err) => {
        console.error("peepthis: set_peep_look failed", err);
        load(prev);
      });
    },
  };
}

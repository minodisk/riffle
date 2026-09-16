const fs = require("fs");
const { execSync } = require("child_process");

const SETTINGS_PATH = ".claude/settings.json";
const LOCAL_PATH = ".claude/settings.local.json";

// Normalize `:*` to ` *` in a Bash permission.
// The official docs say the two are equivalent, but the permission dialog saves
// the space form.
// https://code.claude.com/docs/en/permissions
function normalizePermission(value) {
  if (typeof value !== "string") return value;
  return value.replace(/^(Bash\(.+):(\*\))$/, "$1 $2");
}

// Permission entries to exclude from the merge. The script rejects them so the
// same review feedback does not recur.
// - An over-broad wildcard (e.g. `WebFetch(domain:*)`) effectively allows
//   everything
// - A host-dependent absolute path (e.g. `Read(//private/tmp/...)`,
//   `Read(/Users/...)`, `Bash(bash "/private/tmp/.../scratchpad/foo.sh")`) is
//   invalid or unnecessary elsewhere, so it is not made permanent. A script
//   under the session scratchpad disappears when the session ends, so keeping a
//   Bash entry for it only creates a dead entry
// - `Glob(...)` / `Grep(...)` are invalid syntax. Permissions for those
//   read-only tools are governed by `Read(**)`, so an individual entry is both
//   ineffective and against convention
// - `LS(...)` is not documented explicitly, but like Grep/Glob it is a
//   read-only tool under `Read(**)` and therefore ineffective. Excluded too, to
//   be safe
// - A bare `cd` permission (e.g. `Bash(cd crates/cli)`) only allows changing
//   directory and carries no permission for what follows, so it is noise with
//   no effect
// - A read-only CLI call auto-allowed by a PreToolUse hook needs no entry.
//   Aggregating with a middle wildcard (`Bash(gcloud * list *)`) is not used,
//   because `*` spans whitespace and would allow
//   `gcloud secrets delete list` too
// - `Bash(gh run watch *)` is a duplicate already covered by `Bash(gh run *)`
// - An entry with a script body written inline into an interpreter's `-c` /
//   `-e` (e.g. `Bash(python3 -c "import json,sys; ...")`) only ever matches
//   exactly, and since the same one-liner is never written twice it is only
//   ever a dead entry. It also runs against the policy of putting loops and
//   branches into repository scripts. A shell's `-e` (bash/sh/zsh) is the
//   errexit option rather than an inline script body, so for shells only `-c`
//   is targeted (so `bash -e <script>` is not excluded by mistake)
// - `Write(...)` is redundant because an `Edit(...)` allow/deny applies to the
//   Write tool as well
const EXCLUDED_PATTERNS = [
  /^WebFetch\(domain:\*\)$/,
  /^(?:Read|Edit)\(\/{1,2}(?:private\/tmp|tmp|Users|home|var\/folders)\//,
  /^Bash\((?:[^)]*["'\s=])?\/{1,2}(?:private\/tmp|tmp|Users|home|var\/folders)\//,
  /^(?:Glob|Grep|LS)\(/,
  /^Bash\(cd [^&|;]*\)$/,
  /^Bash\(gcloud [^)]+ (?:list|describe) ?\*?\)$/,
  /^Bash\(gh [^)]+ (?:list|view) ?\*?\)$/,
  /^Bash\(gh run watch ?\*?\)$/,
  /^Bash\((?:python3?|node|deno|ruby|perl) -[ce] /,
  /^Bash\((?:bash|sh|zsh) -c /,
  /^Write\(/,
  /^Skill\(issue\)$/,
];

function isExcluded(value) {
  if (typeof value !== "string") return false;
  return EXCLUDED_PATTERNS.some((re) => re.test(value));
}

function normalizeArray(values) {
  const cleaned = values
    .map(normalizePermission)
    .filter((v) => !isExcluded(v));
  // Strings (permissions) are deduped by value and objects (a hooks matcher
  // block, say) by structure. A Set compares objects by reference, which would
  // fail to reject the same hook arriving as a separate instance per worktree
  // and would multiply it.
  const seen = new Set();
  const deduped = [];
  for (const value of cleaned) {
    const key = typeof value === "string" ? value : JSON.stringify(value);
    if (seen.has(key)) continue;
    seen.add(key);
    deduped.push(value);
  }
  // Sort only arrays of permission strings. Passing object elements (hooks and
  // the like) to sort() would give a meaningless ordering that depends on
  // implicit stringification to "[object Object]", so in that case the original
  // order is preserved.
  if (deduped.every((v) => typeof v === "string")) {
    deduped.sort();
  }
  return deduped;
}

function deepMerge(base, override) {
  if (Array.isArray(override)) {
    const baseArr = Array.isArray(base) ? base : [];
    return normalizeArray([...baseArr, ...override]);
  }
  // Normalize and return base's array only when override is absent.
  // If override is a non-array value (object/primitive), leave it to the logic
  // below as a normal deep merge would, and let override win in the end.
  if (override === undefined && Array.isArray(base)) {
    return normalizeArray(base);
  }
  if (
    base && override && typeof base === "object" && typeof override === "object"
  ) {
    const result = { ...base };
    for (const key of Object.keys(override)) {
      result[key] = deepMerge(base[key], override[key]);
    }
    return Object.fromEntries(
      Object.entries(result).sort(([a], [b]) => a.localeCompare(b)),
    );
  }
  return override ?? base;
}

// `--self` restricts it to the current worktree. That is the path the develop
// skill's `settings-promoter` uses to ride only its own settings.local.json
// along in the archive PR, so unmerged permissions from other worktrees (under
// active development on other branches) are not swept in.
//
// `--write` makes it write `.claude/settings.json` itself instead of printing
// JSON to stdout. This removes the path where an LLM copies hundreds of lines
// of JSON from stdout into the Write tool (a transcription accident actually
// happened there, rewriting an existing entry even though there was nothing to
// promote). The stdout mode is kept for dry checks and backward compatibility.
//
// Unknown arguments are an error. Silently ignoring a typo such as `--sef`
// would drop you into the all-worktrees mode while you thought you were in the
// restricted one, mixing unrelated permissions into the PR.
const argv = process.argv.slice(2);
const KNOWN_ARGS = ["--self", "--write"];
const unknownArgs = argv.filter((a) => !KNOWN_ARGS.includes(a));
if (unknownArgs.length > 0) {
  console.error(`Error: unknown argument(s): ${unknownArgs.join(", ")}`);
  console.error("Usage: merge.js [--self] [--write]");
  process.exit(1);
}
const SELF_ONLY = argv.includes("--self");
const WRITE = argv.includes("--write");

// Get every worktree's path from git worktree list
function getWorktreePaths() {
  const output = execSync("git worktree list --porcelain", {
    encoding: "utf8",
  });
  return output
    .split("\n\n")
    .map((block) => {
      const match = block.match(/^worktree (.+)/m);
      return match ? match[1].trim() : null;
    })
    .filter(Boolean);
}

const base = fs.existsSync(SETTINGS_PATH)
  ? JSON.parse(fs.readFileSync(SETTINGS_PATH, "utf8"))
  : {};

// Merge settings.json and settings.local.json from every worktree
// (only the current one with `--self`)
const worktrees = SELF_ONLY ? ["."] : getWorktreePaths();

// Record which mode it ran in. A forgotten `--self` is otherwise only noticed
// as "a PR with other worktrees' permissions mixed in", so make it visible at
// run time.
console.error(
  SELF_ONLY
    ? "Mode: self-only (current worktree)"
    : `Mode: all worktrees (${worktrees.length})`,
);
let merged = base;
const mergedSettings = [];
const mergedLocals = [];

for (const worktree of worktrees) {
  // Merge settings.json (picking up permissions added on other branches)
  const settingsPath = `${worktree}/.claude/settings.json`;
  if (fs.existsSync(settingsPath)) {
    const settings = JSON.parse(fs.readFileSync(settingsPath, "utf8"));
    merged = deepMerge(merged, settings);
    mergedSettings.push(settingsPath);
  }

  // Merge settings.local.json
  const localPath = `${worktree}/.claude/settings.local.json`;
  if (fs.existsSync(localPath)) {
    const local = JSON.parse(fs.readFileSync(localPath, "utf8"));
    if (Object.keys(local).length > 0) {
      merged = deepMerge(merged, local);
      mergedLocals.push(localPath);
    }
  }
}

if (mergedSettings.length === 0 && mergedLocals.length === 0) {
  console.error(
    `Error: No settings.json or settings.local.json found in any worktree`,
  );
  process.exit(1);
}

console.error(`Merged settings.json: ${mergedSettings.join(", ")}`);
console.error(`Merged settings.local.json: ${mergedLocals.join(", ")}`);

// Pull out only the string permission entries (object elements such as hooks are not counted)
function permissionEntries(settings, key) {
  const values = settings && settings.permissions
    ? settings.permissions[key]
    : undefined;
  if (!Array.isArray(values)) return [];
  return values.filter((v) => typeof v === "string");
}

if (WRITE) {
  // The calling agent reads only this summary to report from (never the JSON itself)
  for (const key of ["allow", "deny", "ask"]) {
    const before = new Set(permissionEntries(base, key));
    const added = permissionEntries(merged, key).filter((v) => !before.has(v));
    console.error(`Added permissions.${key}: ${added.length}`);
  }
  fs.writeFileSync(SETTINGS_PATH, `${JSON.stringify(merged, null, 2)}\n`);
  console.error(`Wrote: ${SETTINGS_PATH}`);
} else {
  console.log(JSON.stringify(merged, null, 2));
}

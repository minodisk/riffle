// The frontend half of the MCP companion: Rust relays a tool call as an
// `mcp-request` event, and the answer is computed here over the main
// window's view state. Free of DOM and Tauri so it is tested without mocks.

import type { BurstMember } from "./burst.js";
import type { PickFlag } from "./selection.js";
import type { SortKey } from "./sort.js";

// The main window's view state, as `main.ts` exposes it.
export interface ViewApi {
  readonly folder: string | null;
  // The files that pass the filter, in strip order.
  readonly files: readonly string[];
  readonly index: number;
  readonly selection: ReadonlySet<string>;
  readonly bursts: ReadonlyMap<string, BurstMember>;
  readonly sharpness: ReadonlyMap<string, number>;
  readonly ratings: ReadonlyMap<string, number>;
  readonly flags: ReadonlyMap<string, "pick" | "reject">;
  readonly labels: ReadonlyMap<string, string>;
  readonly zoomed: boolean;
  readonly comparing: boolean;
  readonly compareActive: string | null;
  readonly sort: SortKey;
  readonly filtered: boolean;
}

export interface McpRequest {
  id: number;
  kind: string;
  args: unknown;
}

export interface McpReply {
  id: number;
  ok: boolean;
  value: unknown;
}

export interface BurstFrame {
  path: string;
  position: number;
  sharpness: number | null;
  rating: number | null;
  flag: PickFlag;
  label: string | null;
}

export interface ViewState {
  folder: string | null;
  count: number;
  current: { path: string; position: number } | null;
  selected: string[];
  mode: "normal" | "zoom" | "compare";
  compare_active: string | null;
  burst: BurstFrame[];
  sort: SortKey;
  filtered: boolean;
}

// The current file's burst in capture order, positions 1-based; empty when
// the file is a burst of its own.
function burstOf(view: ViewApi, path: string): BurstFrame[] {
  const member = view.bursts.get(path);
  if (member === undefined || member.size < 2) return [];
  return [...view.bursts]
    .filter(([, other]) => other.burst === member.burst)
    .sort(([, a], [, b]) => a.position - b.position)
    .map(([other, { position }]) => ({
      path: other,
      position: position + 1,
      sharpness: view.sharpness.get(other) ?? null,
      rating: view.ratings.get(other) ?? null,
      flag: view.flags.get(other) ?? "none",
      label: view.labels.get(other) ?? null,
    }));
}

export function getView(view: ViewApi): ViewState {
  const path = view.files[view.index] as string | undefined;
  return {
    folder: view.folder,
    count: view.files.length,
    current: path === undefined ? null : { path, position: view.index + 1 },
    selected: view.files.filter((file) => view.selection.has(file)),
    mode: view.comparing ? "compare" : view.zoomed ? "zoom" : "normal",
    compare_active: view.comparing ? view.compareActive : null,
    burst: path === undefined ? [] : burstOf(view, path),
    sort: view.sort,
    filtered: view.filtered,
  };
}

export async function handleRequest(kind: string, _args: unknown, view: ViewApi): Promise<unknown> {
  switch (kind) {
    case "get_view":
      return getView(view);
    default:
      throw new Error(`unknown request: ${kind}`);
  }
}

// The reply to `request`, never a rejection: a failure becomes `ok: false`
// with its message, so Rust does not wait for its timeout.
export async function respond(request: McpRequest, view: ViewApi): Promise<McpReply> {
  try {
    const value = await handleRequest(request.kind, request.args, view);
    return { id: request.id, ok: true, value: value ?? null };
  } catch (error) {
    return {
      id: request.id,
      ok: false,
      value: error instanceof Error ? error.message : String(error),
    };
  }
}

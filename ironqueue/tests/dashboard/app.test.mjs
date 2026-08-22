import assert from "node:assert/strict";
import test from "node:test";

import {
  appendCursor,
  compact,
  cursorView,
  dashboardHome,
  dashboardUrl,
  duration,
  entryRequestKey,
  escapeHtml,
  isSuggestionResponseCurrent,
  pageOf,
  parseRoute,
  resetCursor,
} from "../../dashboard/app.mjs";

test("cursor state starts with stable defaults", () => {
  assert.deepEqual(cursorView(), {
    cursor: null,
    history: [],
    start: 1,
    nextCursor: null,
    pageCount: 0,
    limit: 25,
  });
});

test("HTML escaping covers text and attribute metacharacters", () => {
  assert.equal(escapeHtml(`<>&"'`), "&lt;&gt;&amp;&quot;&#39;");
  assert.equal(escapeHtml(42), "42");
});

test("compact numbers switch units without displaying 1000K", () => {
  assert.equal(compact(null), "–");
  assert.equal(compact(9_999), "9999");
  assert.equal(compact(10_000), "10.0K");
  assert.equal(compact(999_949), "999.9K");
  assert.equal(compact(999_950), "1.0M");
});

test("durations use the two largest useful units", () => {
  assert.equal(duration(undefined), "–");
  assert.equal(duration(59_999), "59s");
  assert.equal(duration(61_000), "1m 1s");
  assert.equal(duration(3_661_000), "1h 1m");
  assert.equal(duration(90_000_000), "1d 1h");
});

test("dashboard URLs preserve the mount root", () => {
  assert.equal(dashboardHome(""), "/");
  assert.equal(dashboardHome("/admin"), "/admin");
  assert.equal(dashboardUrl("/admin", "/"), "/admin");
  assert.equal(dashboardUrl("/admin", "/queues/main"), "/admin/queues/main");
});

test("array pagination clamps stale offsets after filtering", () => {
  const view = { offset: 20, limit: 10 };
  assert.deepEqual(pageOf([1, 2, 3], view), [1, 2, 3]);
  assert.equal(view.offset, 0);

  view.offset = 20;
  assert.deepEqual(pageOf(Array.from({ length: 23 }, (_, index) => index), view), [20, 21, 22]);
});

test("cursor helpers reset state and encode a database cursor", () => {
  const view = {
    cursor: { timestamp: "old", id: "old" },
    history: [null],
    start: 26,
    nextCursor: { timestamp: "next", id: "next" },
    pageCount: 25,
  };
  resetCursor(view);
  assert.deepEqual(view, {
    cursor: null,
    history: [],
    start: 1,
    nextCursor: null,
    pageCount: 0,
  });

  const params = new URLSearchParams();
  appendCursor(params, { timestamp: "2026-01-01T00:00:00Z", id: "job-id" }, "cursor_created_at");
  assert.equal(params.get("cursor_created_at"), "2026-01-01T00:00:00Z");
  assert.equal(params.get("cursor_id"), "job-id");
});

test("entry request identity is stable across status insertion order", () => {
  const base = {
    queue: "main",
    name: "mail",
    limit: 25,
    cursor: null,
  };
  const first = entryRequestKey({ ...base, statuses: new Set(["failed", "ready"]) }, "job");
  const second = entryRequestKey({ ...base, statuses: new Set(["ready", "failed"]) }, "job");
  assert.equal(first, second);
  assert.notEqual(first, entryRequestKey({ ...base, statuses: new Set(["ready"]) }, "job"));
});

test("route parsing handles mounts, encoded names, details, and malformed escapes", () => {
  assert.deepEqual(parseRoute("/admin", "/admin"), { view: "home", queue: null, id: null });
  assert.deepEqual(parseRoute("/admin/queues/email%20jobs", "/admin"), {
    view: "queue",
    queue: "email jobs",
    id: null,
  });
  assert.deepEqual(parseRoute("/admin/queues/main/workers/worker-id", "/admin"), {
    view: "worker",
    queue: "main",
    id: "worker-id",
  });
  assert.deepEqual(parseRoute("/queues/main/jobs/job-id", ""), {
    view: "job",
    queue: "main",
    id: "job-id",
  });
  assert.deepEqual(parseRoute("/queues/%ZZ", ""), { view: "queue", queue: "%ZZ", id: null });
  assert.deepEqual(parseRoute("/administrator/queues/main", "/admin"), {
    view: "home",
    queue: null,
    id: null,
  });
});

test("stale suggestion responses cannot replace newer state", () => {
  const view = { suggestionRequest: 4, query: "  email " };
  assert.equal(isSuggestionResponseCurrent(view, 4, "email"), true);
  assert.equal(isSuggestionResponseCurrent(view, 3, "email"), false);
  assert.equal(isSuggestionResponseCurrent(view, 4, "sms"), false);
  assert.equal(isSuggestionResponseCurrent(view, 4), true);
});

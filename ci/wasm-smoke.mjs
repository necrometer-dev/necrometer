#!/usr/bin/env node
// ESM smoke for the wasm-pack output. pkg/seance.js is `"type": "module"`
// and analyze_repos takes (subject, kind, repos_json).
import fs from "node:fs";
import { analyze_repos, initSync, render_card } from "../pkg/seance.js";

const wasmPath = new URL("../pkg/seance_bg.wasm", import.meta.url);
initSync({ module: fs.readFileSync(wasmPath) });

const now = new Date().toISOString();
const repos = JSON.stringify([
  {
    name: "alive",
    pushed_at: now,
    created_at: "2023-01-01T00:00:00Z",
    archived: false,
    fork: false,
    stargazers_count: 5,
    html_url: "https://example.com",
  },
]);

const reading = analyze_repos("ci", "user", repos);
if (!reading.startsWith("{")) {
  throw new Error("bad reading " + reading);
}
const svg = render_card(reading);
if (!svg.startsWith("<svg")) {
  throw new Error("bad svg");
}
console.log("ok reading=" + reading.length + " svg=" + svg.length);

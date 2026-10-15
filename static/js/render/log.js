import { store } from "../store.js";
import { esc } from "../util.js";

function time(ms) {
  const date = new Date(ms);
  return date.toLocaleTimeString("id-ID", { hour12: false }) + "." + String(date.getMilliseconds()).padStart(3, "0");
}

export function renderLog() {
  const log = store.state.log;
  if (!log.length) return `<div class="empty">Belum ada kejadian.</div>`;
  return log
    .map((entry) => `<div class="log-row ${entry.ok ? "" : "fail"}"><time>${time(entry.time_ms)}</time><span>${esc(entry.text)}</span></div>`)
    .join("");
}

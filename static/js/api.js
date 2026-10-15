import { store } from "./store.js";
import { render } from "./render/index.js";
import { $, toast } from "./util.js";

let inflight = 0;

export async function api(method, url, body) {
  inflight++;
  try {
    const res = await fetch(url, {
      method,
      headers: { "Content-Type": "application/json" },
      body: body === undefined ? undefined : JSON.stringify(body),
    });
    const data = await res.json().catch(() => ({}));
    if (!res.ok) throw new Error(data.error || res.statusText);
    store.state = data;
    render();
    return data;
  } finally {
    inflight--;
  }
}

export const run = (task) => task().catch((err) => toast(err.message, true));

export function connect() {
  const events = new EventSource("/api/events");
  events.onmessage = (ev) => {
    if (inflight > 0) return;
    store.state = JSON.parse(ev.data);
    render();
  };
  events.onerror = () => {
    const conn = $("#conn");
    conn.className = "conn off";
    conn.querySelector("span").textContent = "Menyambung ulang ke simulator…";
  };
}

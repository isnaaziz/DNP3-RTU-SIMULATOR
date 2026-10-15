import { store } from "../store.js";
import { $, esc } from "../util.js";

export function renderHeader() {
  const { settings: s, connected, running, error } = store.state;
  $("#endpoint").innerHTML = `<span>${esc(s.bind_ip)}:${s.port}</span><span>Outstation: ${s.outstation_address}</span><span>Master: ${s.master_address}</span><span>Unsolicited: ${s.unsolicited ? "ON" : "OFF"}</span>`;

  const conn = $("#conn");
  conn.className = "conn " + (connected ? "on" : running ? "wait" : "off");
  conn.querySelector("span").textContent = connected ? "Master terhubung" : running ? "Menunggu master" : "Outstation berhenti";

  const alert = $("#error");
  alert.textContent = error || "";
  alert.classList.toggle("hidden", !error);
}

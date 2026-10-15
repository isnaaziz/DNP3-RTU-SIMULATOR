import { api } from "../api.js";
import { store } from "../store.js";
import { $, toast } from "../util.js";

const form = () => $("#settings-form");

function open() {
  const { settings, locked_port: locked } = store.state;
  const e = form().elements;
  for (const [key, value] of Object.entries(settings)) {
    const el = e[key];
    if (!el) continue;
    if (el.type === "checkbox") el.checked = value;
    else el.value = value;
  }
  e.port.readOnly = locked != null;
  e.port.title = locked != null ? `Port DNP3 dikunci ${locked} oleh --dnp3-port (mapping Docker/Kubernetes)` : "";
  const note = $("#port-note");
  note.classList.toggle("hidden", locked == null);
  note.textContent = locked != null ? `Port DNP3 dikunci di ${locked} oleh --dnp3-port, sesuai port yang dibuka Docker/Kubernetes.` : "";
  $("#settings-dialog").showModal();
}

function submit(ev) {
  ev.preventDefault();
  const e = form().elements;
  const body = {
    bind_ip: e.bind_ip.value.trim(),
    port: Number(e.port.value),
    outstation_address: Number(e.outstation_address.value),
    master_address: Number(e.master_address.value),
    event_buffer: Number(e.event_buffer.value),
    unsolicited: e.unsolicited.checked,
  };
  api("POST", "/api/settings", body)
    .then(() => {
      $("#settings-dialog").close();
      toast("Outstation di-restart");
    })
    .catch((err) => toast(err.message, true));
}

export function initSettingsDialog() {
  $("#settings-btn").addEventListener("click", open);
  form().addEventListener("submit", submit);
}

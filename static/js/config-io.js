import { api, run } from "./api.js";
import { $, toast } from "./util.js";

async function importFile(ev) {
  const file = ev.target.files[0];
  ev.target.value = "";
  if (!file) return;

  let data;
  try {
    data = JSON.parse(await file.text());
  } catch {
    return toast(`${file.name} bukan file JSON yang valid`, true);
  }

  const count = Array.isArray(data.points) ? data.points.length : 0;
  const message = `Import ${count} point dari ${file.name}?\nSemua point dan pengaturan saat ini akan diganti, dan koneksi master terputus sebentar.`;
  if (!confirm(message)) return;
  run(async () => {
    await api("POST", "/api/config/import", data);
    toast(`${count} point diimport`);
  });
}

export function initConfigIo() {
  $("#export-btn").addEventListener("click", () => (window.location.href = "/api/config/export"));
  $("#template-btn").addEventListener("click", () => (window.location.href = "/api/config/template"));
  $("#import-btn").addEventListener("click", () => $("#import-file").click());
  $("#import-file").addEventListener("change", importFile);
}

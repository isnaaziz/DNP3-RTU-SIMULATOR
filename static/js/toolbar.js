import { api, run } from "./api.js";
import { openPoint } from "./dialogs/point-dialog.js";
import { $, toast } from "./util.js";

function toggleFullscreen() {
  if (document.fullscreenElement) return document.exitFullscreen();
  document.documentElement.requestFullscreen().catch(() => toast("Layar penuh tidak diizinkan browser", true));
}

export function initToolbar() {
  $("#add-btn").addEventListener("click", () => openPoint(null));
  $("#fullscreen-btn").addEventListener("click", toggleFullscreen);
  $("#log-btn").addEventListener("click", () => $("#drawer").classList.toggle("open"));
  $("#close-log").addEventListener("click", () => $("#drawer").classList.remove("open"));
  $("#clear-log").addEventListener("click", () => run(() => api("DELETE", "/api/log")));
}

import { api, run } from "../api.js";
import { deletePoint, ensureLinks } from "../actions.js";
import { FLAGS, KIND, nextIndex, store } from "../store.js";
import { $, toast } from "../util.js";

const form = () => $("#point-form");
const dialog = () => $("#point-dialog");
const isOutput = (kind) => kind === "binary_output" || kind === "analog_output";

const FEEDBACK_OPTIONS = {
  binary_output: [["double_bit", "Double (DBI)"], ["binary_input", "Single (BI)"], ["", "Tanpa status"]],
  analog_output: [["analog_input", "Analog Input (AI)"], ["", "Tanpa feedback"]],
};

function syncLr() {
  const hasLr = form().elements.lr_kind.value !== "";
  $("#lr-index-label").classList.toggle("hidden", !hasLr);
  $("#local-label").classList.toggle("hidden", hasLr);
}

function syncKind() {
  const kind = form().elements.kind.value;
  $("#fb-set").classList.toggle("hidden", !isOutput(kind));
  $("#sim-set").classList.toggle("hidden", isOutput(kind));
  $("#close-index-label").classList.toggle("hidden", kind !== "binary_output");

  const select = form().elements.fb_kind;
  const options = FEEDBACK_OPTIONS[kind] || FEEDBACK_OPTIONS.analog_output;
  const current = select.value;
  select.innerHTML = options.map(([value, text]) => `<option value="${value}">${text}</option>`).join("");
  if (options.some(([value]) => value === current)) select.value = current;
  syncLr();
}

function fill(point) {
  const e = form().elements;
  e.kind.value = point.kind;
  syncKind();
  e.index.value = point.index;
  e.name.value = point.name;
  e.unit.value = point.unit || "";
  e.value.value = point.value;
  e.class.value = point.class;
  e.sim_mode.value = point.sim ? point.sim.mode : "";
  if (point.sim) {
    e.sim_min.value = point.sim.min;
    e.sim_max.value = point.sim.max;
    e.sim_period.value = point.sim.period_s;
  }
  e.fb_kind.value = point.feedback ? point.feedback.kind : "";
  if (point.feedback) {
    e.fb_index.value = point.feedback.index;
    e.fb_travel.value = point.feedback.travel_ms;
  }
  e.close_index.value = point.close_index ?? "";
  e.local.checked = point.local;
  e.lr_kind.value = point.lr ? point.lr.kind : "";
  if (point.lr) e.lr_index.value = point.lr.index;
  syncLr();
}

function fillNew() {
  const e = form().elements;
  syncKind();
  e.index.value = nextIndex(e.kind.value);
  e.fb_index.value = nextIndex(e.fb_kind.value || "double_bit");
  e.lr_index.value = nextIndex("binary_input");
  syncLr();
}

function readForm() {
  const e = form().elements;
  const kind = e.kind.value;
  const output = isOutput(kind);
  const flags = [...form().querySelectorAll("[name=flag]:checked")].reduce((acc, box) => acc | Number(box.value), 0);
  return {
    kind,
    index: Number(e.index.value),
    name: e.name.value,
    unit: e.unit.value,
    value: Number(e.value.value || 0),
    class: Number(e.class.value),
    flags,
    local: output && !e.lr_kind.value && e.local.checked,
    lr: output && e.lr_kind.value ? { kind: e.lr_kind.value, index: Number(e.lr_index.value) } : null,
    sim:
      !output && e.sim_mode.value
        ? { mode: e.sim_mode.value, min: Number(e.sim_min.value), max: Number(e.sim_max.value), period_s: Number(e.sim_period.value) }
        : null,
    feedback:
      output && e.fb_kind.value
        ? { kind: e.fb_kind.value, index: Number(e.fb_index.value), travel_ms: Number(e.fb_travel.value) }
        : null,
    close_index: kind === "binary_output" && e.close_index.value !== "" ? Number(e.close_index.value) : null,
  };
}

export function openPoint(point) {
  store.editing = point || null;
  form().reset();
  $("#point-title").textContent = point ? `Ubah ${KIND[point.kind]} ${point.index} — ${point.name}` : "Tambah point";
  $("#delete-point").classList.toggle("hidden", !point);
  if (point) fill(point);
  else fillNew();
  const flags = point ? point.flags : 0x01;
  for (const box of form().querySelectorAll("[name=flag]")) box.checked = (flags & Number(box.value)) !== 0;
  dialog().showModal();
}

export function initPointDialog() {
  $("#flag-boxes").innerHTML = FLAGS.map(
    ([bit, name]) => `<label class="check"><input type="checkbox" name="flag" value="${bit}"> ${name}</label>`,
  ).join("");

  const e = form().elements;
  e.kind.addEventListener("change", () => {
    syncKind();
    if (!store.editing) e.index.value = nextIndex(e.kind.value);
  });
  e.lr_kind.addEventListener("change", () => {
    syncLr();
    if (e.lr_kind.value) e.lr_index.value = nextIndex(e.lr_kind.value);
  });

  form().addEventListener("submit", (ev) => {
    ev.preventDefault();
    const body = readForm();
    const editing = store.editing;
    const request = editing ? api("PUT", `/api/points/${editing.id}`, body) : api("POST", "/api/points", body);
    request
      .then(async () => {
        dialog().close();
        await ensureLinks(body);
      })
      .catch((err) => toast(err.message, true));
  });

  $("#delete-point").addEventListener("click", () => {
    const point = store.editing;
    if (!point) return;
    dialog().close();
    run(() => deletePoint(point));
  });
}

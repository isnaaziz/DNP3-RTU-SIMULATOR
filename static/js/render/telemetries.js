import { ICON } from "../icons.js";
import { step, store } from "../store.js";
import { esc, fmt } from "../util.js";
import { editable, emptyState, qualityText, rowActions } from "./shared.js";

const ORDER = { analog_input: 0, analog_output: 1, counter: 2 };
const SUBTITLE = { analog_input: "AI", analog_output: "AO · setpoint", counter: "Counter" };

function value(point) {
  if (store.editingValue === point.id) {
    return `<input class="val-input" type="number" step="any" value="${point.value}" data-value-input="${point.id}">`;
  }
  const unit = point.unit ? `<small>${esc(point.unit)}</small>` : "";
  return `<button class="val-btn" data-value-edit="${point.id}" title="Klik untuk isi nilai">${fmt(point.value)}${unit}</button>`;
}

function row(point) {
  const auto =
    point.kind === "analog_output"
      ? `<span style="width:40px"></span>`
      : `<button class="auto ${point.sim ? "active" : ""}" data-auto="${point.id}" title="Simulasi otomatis">A</button>`;
  return `
    <div class="row">
      <div class="name"><button data-edit="${point.id}">${esc(point.name)}</button><small>${SUBTITLE[point.kind]}</small>${rowActions(point)}</div>
      <div>${value(point)}<div class="idx">Index:${editable(point, "index", point.index)}</div>${qualityText(point)}</div>
      <div class="btns">
        <div class="pm-stack">
          <button class="pm" data-step="${point.id}" data-dir="1" title="Tambah ${step(point)}">+</button>
          <button class="pm" data-step="${point.id}" data-dir="-1" title="Kurangi ${step(point)}">−</button>
        </div>
        ${auto}
      </div>
    </div>`;
}

export function renderTelemetries() {
  const values = store.state.points
    .filter((p) => p.kind in ORDER)
    .sort((a, b) => ORDER[a.kind] - ORDER[b.kind] || a.index - b.index);
  if (!values.length) return emptyState(ICON.chart, "Belum ada data telemetry");
  return `<div class="list">${values.map(row).join("")}</div>`;
}

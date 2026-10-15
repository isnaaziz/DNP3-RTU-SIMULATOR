import { ICON } from "../icons.js";
import { KIND, position, statusLinks, store } from "../store.js";
import { esc } from "../util.js";
import { editable, emptyState, qualityText, rowActions } from "./shared.js";

const DOUBLE_TEXT = { open: "OPEN", close: "CLOSE", inter: "INTER", bad: "BAD" };

const autoButton = (point) =>
  `<button class="auto ${point.sim ? "active" : ""}" data-auto="${point.id}" title="Simulasi otomatis">A</button>`;

function doubleControls(point) {
  const button = (value, label, activeClass) =>
    `<button class="sq ${point.value === value ? activeClass : ""}" data-set="${point.id}" data-v="${value}">${label}</button>`;
  return `${autoButton(point)}
    <div class="dbi-btns">
      ${button(1, "OPEN", "open-on")}
      ${button(2, "CLOSE", "close-on")}
      ${button(0, "00", "amber-on")}
      ${button(3, "11", "amber-on")}
    </div>`;
}

function singleControls(point) {
  const on = Boolean(point.value);
  return `<div class="combo">${autoButton(point)}<button class="state ${on ? "on" : ""}" data-set="${point.id}" data-v="${on ? 0 : 1}" title="Klik untuk ${on ? "OFF" : "ON"}">${on ? "ON" : "OFF"}</button></div>`;
}

function row(point) {
  const isDouble = point.kind === "double_bit";
  const pos = position(point);
  const text = isDouble ? DOUBLE_TEXT[pos] : point.value ? "ON" : "OFF";
  const cls = isDouble ? pos : point.value ? "on" : "off";
  return `
    <div class="row">
      <div class="name"><button data-edit="${point.id}">${esc(point.name)}</button><small>${KIND[point.kind]}</small>${rowActions(point)}</div>
      <div><div class="big ${cls}">${text}</div><div class="idx">Index:${editable(point, "index", point.index)}</div>${qualityText(point)}</div>
      <div class="btns">${isDouble ? doubleControls(point) : singleControls(point)}</div>
    </div>`;
}

export function renderTelesignals() {
  const linked = statusLinks();
  const signals = store.state.points.filter(
    (p) => (p.kind === "binary_input" || p.kind === "double_bit") && !linked.has(`${p.kind}:${p.index}`),
  );
  if (!signals.length) return emptyState(ICON.signal, "Belum ada data telesignal");
  return `<div class="ts-list">${signals.map(row).join("")}</div>`;
}

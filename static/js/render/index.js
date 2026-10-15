import { store } from "../store.js";
import { $ } from "../util.js";
import { renderBreakers } from "./breakers.js";
import { renderHeader } from "./header.js";
import { renderLog } from "./log.js";
import { renderTelemetries } from "./telemetries.js";
import { renderTelesignals } from "./telesignals.js";

const painted = {};
let pointerDown = false;
let pending = false;

function paint(selector, html) {
  if (painted[selector] === html) return;
  const el = $(selector);
  const active = document.activeElement;
  if (active && active.tagName === "INPUT" && el.contains(active)) return;
  el.innerHTML = html;
  painted[selector] = html;
}

export function render() {
  if (!store.state) return;
  if (pointerDown) {
    pending = true;
    return;
  }
  renderHeader();
  paint("#cb", renderBreakers());
  paint("#ts", renderTelesignals());
  paint("#tm", renderTelemetries());
  paint("#log", renderLog());

  const input = document.querySelector("[data-value-input], [data-inline-input]");
  if (input && document.activeElement !== input) {
    input.focus();
    input.select();
  }
}

export function installPointerGuard() {
  const release = () =>
    setTimeout(() => {
      pointerDown = false;
      if (pending) {
        pending = false;
        render();
      }
    }, 0);
  document.addEventListener("pointerdown", () => (pointerDown = true), true);
  document.addEventListener("pointerup", release, true);
  document.addEventListener("pointercancel", release, true);
}

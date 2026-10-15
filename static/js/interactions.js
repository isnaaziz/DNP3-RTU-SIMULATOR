import { run } from "./api.js";
import { deletePoint, savePoint, setValue, toggleLocal, updatePoint } from "./actions.js";
import { openPoint } from "./dialogs/point-dialog.js";
import { render } from "./render/index.js";
import { byId, defaultSim, freeBoIndex, nextIndex, step, store } from "./store.js";

function commitInline(input) {
  const point = byId(input.dataset.inlineInput);
  const path = input.dataset.path;
  store.inlineEdit = null;
  if (!point || input.value === "") return render();
  input.blur();
  updatePoint(point, (data) => {
    const keys = path.split(".");
    let target = data;
    for (const key of keys.slice(0, -1)) target = target[key];
    target[keys[keys.length - 1]] = Number(input.value);
  });
}

function commitValue(input) {
  const point = byId(input.dataset.valueInput);
  store.editingValue = null;
  if (!point || input.value === "") return render();
  input.blur();
  setValue(point, Number(input.value));
}

function cancelEditing() {
  store.editingValue = null;
  store.inlineEdit = null;
  render();
}

const handlers = {
  set: (d) => setValue(byId(d.set), Number(d.v)),
  edit: (d) => openPoint(byId(d.edit)),
  del: (d) => run(() => deletePoint(byId(d.del))),
  local: (d) => toggleLocal(byId(d.local)),
  auto: (d) => {
    const point = byId(d.auto);
    savePoint(point, { sim: point.sim ? null : defaultSim(point) });
  },
  step: (d) => {
    const point = byId(d.step);
    const next = +(point.value + Number(d.dir) * step(point)).toFixed(6);
    setValue(point, point.kind === "counter" ? Math.max(0, next) : next);
  },
  valueEdit: (d) => {
    store.editingValue = Number(d.valueEdit);
    store.inlineEdit = null;
    render();
  },
  inline: (d) => {
    store.inlineEdit = { id: Number(d.inline), path: d.path };
    store.editingValue = null;
    render();
  },
  ctl: (d) => {
    const point = byId(d.ctl);
    const close = d.mode === "2" ? point.close_index ?? freeBoIndex(point.index) : null;
    updatePoint(point, (data) => (data.close_index = close));
  },
  addLr: (d) => {
    updatePoint(byId(d.addLr), (data) => {
      data.lr = { kind: "binary_input", index: nextIndex("binary_input") };
      data.local = false;
    });
  },
};

export function installInteractions() {
  document.addEventListener("click", (ev) => {
    const button = ev.target.closest("button");
    if (!button) return;
    const data = button.dataset;
    if ("close" in data) return button.closest("dialog").close();
    const action = Object.keys(handlers).find((key) => data[key] !== undefined);
    if (action) handlers[action](data);
  });

  document.addEventListener("keydown", (ev) => {
    const data = ev.target.dataset;
    if (!data || !(data.valueInput || data.inlineInput)) return;
    if (ev.key === "Enter") return data.inlineInput ? commitInline(ev.target) : commitValue(ev.target);
    if (ev.key === "Escape") {
      ev.target.blur();
      cancelEditing();
    }
  });

  document.addEventListener("focusout", (ev) => {
    const data = ev.target.dataset || {};
    if (!data.valueInput && !data.inlineInput) return;
    setTimeout(() => {
      if (document.activeElement && document.activeElement.tagName === "INPUT") return;
      if (store.editingValue !== null || store.inlineEdit !== null) cancelEditing();
    }, 150);
  });
}

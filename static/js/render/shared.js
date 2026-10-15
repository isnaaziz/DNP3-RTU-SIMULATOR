import { ICON } from "../icons.js";
import { FLAGS, store } from "../store.js";
import { esc } from "../util.js";

export function editable(point, path, value, suffix = "") {
  const edit = store.inlineEdit;
  if (edit && edit.id === point.id && edit.path === path) {
    return `<input class="inline-input" type="number" min="0" step="1" value="${value}" data-inline-input="${point.id}" data-path="${path}">`;
  }
  return `<button class="inline-val" data-inline="${point.id}" data-path="${path}" title="Klik untuk ubah">${value}${suffix}</button>`;
}

export function qualityText(point) {
  const bad = [];
  if (!(point.flags & 0x01)) bad.push("INVALID");
  for (const [bit, name] of FLAGS.slice(1)) if (point.flags & bit) bad.push(name);
  return bad.length ? `<div class="quality">${bad.join(" · ")}</div>` : "";
}

export function rowActions(point) {
  return `<div class="row-actions">
    <button class="mini-icon" data-edit="${point.id}" title="Ubah" aria-label="Ubah ${esc(point.name)}">${ICON.pencil}</button>
    <button class="mini-icon danger" data-del="${point.id}" title="Hapus" aria-label="Hapus ${esc(point.name)}">${ICON.trash}</button>
  </div>`;
}

export const emptyState = (icon, text) => `<div class="empty">${icon}<div>${text}</div></div>`;

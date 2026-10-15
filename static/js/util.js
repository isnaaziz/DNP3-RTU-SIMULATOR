export const $ = (selector) => document.querySelector(selector);

export const esc = (text) =>
  String(text).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));

export function fmt(value) {
  if (Number.isInteger(value)) return String(value);
  const abs = Math.abs(value);
  return value.toFixed(abs >= 100 ? 1 : abs >= 10 ? 2 : 3).replace(/0+$/, "").replace(/\.$/, "");
}

export function toast(message, error = false) {
  const el = $("#toast");
  el.textContent = message;
  el.className = "show" + (error ? " error" : "");
  clearTimeout(el._hide);
  el._hide = setTimeout(() => (el.className = ""), 3200);
}

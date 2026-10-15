import { ICON } from "../icons.js";
import { KIND, find, isLocal, lrPoint, position, store } from "../store.js";
import { esc } from "../util.js";
import { editable, emptyState, qualityText } from "./shared.js";

const LOCK_TITLE = "Mode REMOTE: CB hanya bisa dioperasikan dari master. Pindah ke LOCAL untuk operasi dari panel.";
const LABELS = { open: "OPEN", close: "CLOSE", inter: "INTERMEDIATE", bad: "BAD STATE" };

function lamps(bo, status, pos, online) {
  const isDouble = status.kind === "double_bit";
  const remote = !isLocal(bo);
  const lit = (color) => {
    if (!online) return "dead";
    const on = (color === "green" && (pos === "open" || pos === "bad")) || (color === "red" && (pos === "close" || pos === "bad"));
    return on ? "lit" : "";
  };
  const attrs = (value, label) =>
    remote ? `disabled title="${LOCK_TITLE}"` : `data-set="${status.id}" data-v="${value}" title="Operasi lokal: ${label}"`;

  return `
    <div class="lamps ${pos === "inter" && online ? "inter" : ""}">
      <button class="lamp-btn" ${attrs(isDouble ? 1 : 0, "OPEN")}><span class="lamp green ${lit("green")}">${ICON.open}</span><span class="lamp-label green">Open</span></button>
      <button class="lamp-btn" ${attrs(isDouble ? 2 : 1, "CLOSE")}><span class="lamp red ${lit("red")}">${ICON.close}</span><span class="lamp-label red">Close</span></button>
      ${isDouble ? `<div class="small-row">
        <button class="chip-btn inter ${pos === "inter" ? "sel" : ""}" data-set="${status.id}" data-v="0">Inter 00</button>
        <button class="chip-btn ${pos === "bad" ? "sel" : ""}" data-set="${status.id}" data-v="3">Bad 11</button>
      </div>` : ""}
      <div class="state-text ${online ? pos : "dead"}">${online ? LABELS[pos] : "INVALID"}</div>
      ${remote ? `<div class="lock-note" title="${LOCK_TITLE}">${ICON.lock}Remote: operasi lokal terkunci</div>` : ""}
    </div>`;
}

function info(bo, status, feedback) {
  const isDouble = status.kind === "double_bit";
  const mode = !bo.feedback ? "Kontrol saja" : isDouble ? "Double status &amp; kontrol" : "Single status &amp; kontrol";
  const missing = (found) => (found ? "" : ` <span class="quality">tidak ditemukan</span>`);
  const local = isLocal(bo);
  const single = bo.close_index == null;

  const statusRow = bo.feedback
    ? `<div>Index status ${KIND[bo.feedback.kind]}:${editable(bo, "feedback.index", bo.feedback.index)}${missing(feedback)}</div>`
    : "";
  const controlRows = single
    ? `<div>Index kontrol BO:${editable(bo, "index", bo.index)}</div>`
    : `<div>Index kontrol Open:${editable(bo, "index", bo.index)}</div><div>Index kontrol Close:${editable(bo, "close_index", bo.close_index)}</div>`;
  const lrRow = bo.lr
    ? `<div>Index L/R ${KIND[bo.lr.kind]}:${editable(bo, "lr.index", bo.lr.index)}${missing(lrPoint(bo))}</div>`
    : `<div>Index L/R:<button class="link-btn" data-add-lr="${bo.id}">+ buat point L/R</button> <span class="meta">(sekarang internal)</span></div>`;

  return `
    <div class="info">
      ${statusRow}
      ${controlRows}
      <div>Tipe kontrol:<span class="seg"><button class="${single ? "sel" : ""}" data-ctl="${bo.id}" data-mode="1" title="Satu index BO, arah dari kode TRIP/CLOSE atau LATCH ON/OFF">1 index</button><button class="${single ? "" : "sel"}" data-ctl="${bo.id}" data-mode="2" title="Index BO terpisah untuk Open dan Close">2 index</button></span></div>
      ${bo.feedback ? `<div>Waktu tempuh:${editable(bo, "feedback.travel_ms", bo.feedback.travel_ms, " ms")}</div>` : ""}
      <div>Status BO:<b>${bo.value ? "ON" : "OFF"}</b></div>
      <div>Mode: ${mode}</div>
      ${lrRow}
      ${qualityText(status)}
      <div class="lr">
        <span>L</span>
        <button class="switch ${local ? "" : "remote"}" data-local="${bo.id}" title="${local ? "LOCAL: kontrol dari master ditolak" : "REMOTE: kontrol dari master diterima"}" aria-label="Local/Remote"></button>
        <span>R</span>
        <span class="tag">${local ? "Local" : "Remote"}</span>
      </div>
    </div>`;
}

function bay(bo) {
  const feedback = bo.feedback ? find(bo.feedback.kind, bo.feedback.index) : null;
  const status = feedback || bo;
  const pos = position(status);
  const online = (status.flags & 0x01) !== 0;
  return `
    <article class="bay">
      <h3 class="bay-title">
        <span class="bay-icon ${online ? pos : "dead"}">${ICON.breaker}</span>
        <button data-edit="${bo.id}">${esc(bo.name)}</button>
        <button class="bay-edit" data-edit="${bo.id}" title="Ubah semua pengaturan" aria-label="Ubah ${esc(bo.name)}">${ICON.pencil}</button>
        <button class="bay-edit danger" data-del="${bo.id}" title="Hapus" aria-label="Hapus ${esc(bo.name)}">${ICON.trash}</button>
      </h3>
      <div class="bay-body">
        ${lamps(bo, status, pos, online)}
        ${info(bo, status, feedback)}
      </div>
    </article>`;
}

export function renderBreakers() {
  const outputs = store.state.points.filter((p) => p.kind === "binary_output");
  if (!outputs.length) {
    return emptyState(ICON.breaker, "Belum ada circuit breaker.<br>Klik + lalu pilih tipe “Circuit Breaker / kontrol”.");
  }
  return `<div class="bays">${outputs.map(bay).join("")}</div>`;
}

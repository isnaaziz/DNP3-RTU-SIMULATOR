import { api, run } from "./api.js";
import { render } from "./render/index.js";
import { KIND, find, linkedPoints, lrPoint, isLocal, pointData } from "./store.js";
import { toast } from "./util.js";

export function setValue(point, value) {
  point.value = value;
  render();
  run(() => api("POST", `/api/points/${point.id}/value`, { value }));
}

export function savePoint(point, patch) {
  const body = pointData(point, patch);
  Object.assign(point, patch);
  render();
  run(() => api("PUT", `/api/points/${point.id}`, body));
}

export async function ensureLinks(body) {
  const created = [];
  const ensure = async (link, name) => {
    if (!link || find(link.kind, link.index)) return;
    await api("POST", "/api/points", {
      kind: link.kind,
      index: link.index,
      name,
      value: link.kind === "double_bit" ? 1 : 0,
      class: body.class ?? 1,
      flags: 0x01,
    });
    created.push(`${KIND[link.kind]} ${link.index}`);
  };
  const feedback = body.feedback;
  await ensure(feedback && feedback.kind !== "analog_input" ? feedback : null, body.name);
  await ensure(feedback && feedback.kind === "analog_input" ? feedback : null, `${body.name} Feedback`);
  await ensure(body.lr, `${body.name} L/R`);
  if (created.length) toast(`Point ${created.join(" dan ")} ikut dibuat`);
}

export function updatePoint(point, mutate) {
  const data = structuredClone(pointData(point));
  mutate(data);
  run(async () => {
    await api("PUT", `/api/points/${point.id}`, data);
    await ensureLinks(data);
  });
}

export async function deletePoint(point) {
  if (!confirm(`Hapus ${KIND[point.kind]} ${point.index} "${point.name}"?`)) return;
  const linked = linkedPoints(point);
  const list = linked.map((q) => `• ${KIND[q.kind]} ${q.index} "${q.name}"`).join("\n");
  const withLinked = linked.length > 0 && confirm(`Hapus juga point terkait?\n${list}`);
  await api("DELETE", `/api/points/${point.id}`);
  if (withLinked) for (const q of linked) await api("DELETE", `/api/points/${q.id}`);
  toast(withLinked ? `${linked.length + 1} point dihapus` : "Point dihapus");
}

export function toggleLocal(point) {
  const lr = lrPoint(point);
  const local = !isLocal(point);
  if (!lr) return savePoint(point, { local });
  setValue(lr, lr.kind === "double_bit" ? (local ? 2 : 1) : local ? 1 : 0);
}

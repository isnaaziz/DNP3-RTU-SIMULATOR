export const KIND = {
  binary_input: "BI",
  double_bit: "DBI",
  binary_output: "BO",
  analog_input: "AI",
  analog_output: "AO",
  counter: "Counter",
};

export const FLAGS = [
  [0x01, "ONLINE"],
  [0x02, "RESTART"],
  [0x04, "COMM_LOST"],
  [0x08, "REMOTE_FORCED"],
  [0x10, "LOCAL_FORCED"],
];

export const store = {
  state: null,
  editing: null,
  editingValue: null,
  inlineEdit: null,
};

const points = () => store.state.points;

export const byId = (id) => points().find((p) => p.id === Number(id));

export const find = (kind, index) => points().find((p) => p.kind === kind && p.index === index);

export function pointData(point, patch = {}) {
  const { id, ...data } = point;
  return { ...data, ...patch };
}

export function nextIndex(kind) {
  const used = points().filter((p) => p.kind === kind).map((p) => p.index);
  return used.length ? Math.max(...used) + 1 : 0;
}

export function freeBoIndex(from) {
  const used = new Set(
    points()
      .filter((p) => p.kind === "binary_output")
      .flatMap((p) => [p.index, p.close_index].filter((v) => v != null)),
  );
  let index = from + 1;
  while (used.has(index)) index++;
  return index;
}

export const lrPoint = (point) => (point.lr ? find(point.lr.kind, point.lr.index) : null);

export function isLocal(point) {
  const lr = lrPoint(point);
  if (!lr) return point.local;
  return lr.kind === "double_bit" ? lr.value === 2 : lr.value !== 0;
}

export function position(point) {
  if (!point) return null;
  if (point.kind === "double_bit") return ["inter", "open", "close", "bad"][point.value] || "bad";
  return point.value ? "close" : "open";
}

export function step(point) {
  if (point.kind === "counter") return 1;
  const abs = Math.abs(point.value);
  return abs >= 1000 ? 10 : abs >= 100 ? 1 : abs >= 1 ? 0.1 : 0.01;
}

export function defaultSim(point) {
  if (point.kind === "binary_input" || point.kind === "double_bit") return { mode: "toggle", min: 0, max: 1, period_s: 30 };
  if (point.kind === "counter") return { mode: "random", min: 1, max: 5, period_s: 1 };
  const span = Math.abs(point.value) * 0.05 || 5;
  return { mode: "random", min: +(point.value - span).toFixed(3), max: +(point.value + span).toFixed(3), period_s: 60 };
}

export function linkedPoints(point) {
  const others = points().filter((q) => q.id !== point.id);
  const usedElsewhere = (link) =>
    others.some((q) => [q.feedback, q.lr].some((l) => l && l.kind === link.kind && l.index === link.index));
  return [point.feedback, point.lr]
    .filter((link) => link && !usedElsewhere(link))
    .map((link) => find(link.kind, link.index))
    .filter(Boolean);
}

export function statusLinks() {
  return new Set(points().flatMap((p) => [p.feedback, p.lr].filter(Boolean).map((l) => `${l.kind}:${l.index}`)));
}

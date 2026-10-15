const svg = (viewBox, width, body) =>
  `<svg viewBox="${viewBox}" fill="none" stroke="currentColor" stroke-width="${width}" stroke-linecap="round" stroke-linejoin="round">${body}</svg>`;

export const ICON = {
  open: svg("0 0 24 24", 2.4, `<path d="M12 2.5v6M12 15.5v6M12 15.5L6.5 9.5"/><circle cx="12" cy="8.5" r="1.2" fill="currentColor"/>`),
  close: svg("0 0 24 24", 2.4, `<path d="M12 2.5v19"/><circle cx="12" cy="8.5" r="1.2" fill="currentColor"/><path d="M9.5 15.5h5"/>`),
  breaker: svg("0 0 24 24", 2.2, `<path d="M12 2v5M12 17v5"/><rect x="7" y="7" width="10" height="10" rx="1"/>`),
  signal: svg("0 0 48 48", 2.6, `<path d="M14 12a17 17 0 0 0 0 24M34 12a17 17 0 0 1 0 24M19 17a10 10 0 0 0 0 14M29 17a10 10 0 0 1 0 14M21 28l6-8"/>`),
  chart: svg("0 0 24 24", 2, `<path d="M3 3v18h18"/><path d="M6 15l4-5 4 3 6-7"/>`),
  pencil: svg("0 0 24 24", 2, `<path d="M4 20h4L18.5 9.5a2.1 2.1 0 0 0-4-4L4 16v4zM13.5 6.5l4 4"/>`),
  trash: svg("0 0 24 24", 2, `<path d="M4 7h16M10 11v6M14 11v6M6 7l1 13h10l1-13M9 7V4h6v3"/>`),
  lock: svg("0 0 24 24", 2, `<rect x="5" y="11" width="14" height="10" rx="2"/><path d="M8 11V7a4 4 0 0 1 8 0v4"/>`),
};

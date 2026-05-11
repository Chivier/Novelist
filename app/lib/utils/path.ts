export function pathBasename(path: string): string {
  const normalized = path.replace(/[\\/]+$/, '');
  const idx = Math.max(normalized.lastIndexOf('/'), normalized.lastIndexOf('\\'));
  return idx >= 0 ? normalized.slice(idx + 1) : normalized;
}

export function pathDirname(path: string): string {
  const normalized = path.replace(/[\\/]+$/, '');
  const idx = Math.max(normalized.lastIndexOf('/'), normalized.lastIndexOf('\\'));
  return idx > 0 ? normalized.slice(0, idx) : '';
}

export function pathJoin(dir: string, name: string): string {
  if (!dir) return name;
  const sep = dir.includes('\\') && !dir.includes('/') ? '\\' : '/';
  return `${dir.replace(/[\\/]+$/, '')}${sep}${name.replace(/^[\\/]+/, '')}`;
}

export function pathStartsWithChild(path: string, parent: string): boolean {
  let normalizedPath = path.replace(/\\/g, '/');
  let normalizedParent = parent.replace(/\\/g, '/').replace(/\/+$/, '');
  const windowsStyle = /[A-Za-z]:/.test(path) || /[A-Za-z]:/.test(parent) || path.includes('\\') || parent.includes('\\');
  if (windowsStyle) {
    normalizedPath = normalizedPath.toLowerCase();
    normalizedParent = normalizedParent.toLowerCase();
  }
  return normalizedPath.startsWith(`${normalizedParent}/`);
}

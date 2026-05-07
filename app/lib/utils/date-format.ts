/**
 * Filename-safe date / time formatter for the `{date:fmt}` `{time:fmt}`
 * `{datetime:fmt}` macro family. Pure, no allocations beyond the result
 * string, deliberately lightweight (no dayjs/luxon).
 *
 * Supported tokens:
 *   YYYY  4-digit year         (2026)
 *   YY    2-digit year         (26)
 *   MM    2-digit month        (05)
 *   M     1-or-2-digit month   (5)
 *   DD    2-digit day          (07)
 *   D     1-or-2-digit day     (7)
 *   HH    2-digit hour 24      (14)
 *   mm    2-digit minute       (08)
 *   ss    2-digit second       (32)
 *
 * Reserved keywords (entire fmt string):
 *   timestamp   → Unix seconds (UTC)
 *   iso         → ISO 8601 with local offset, e.g. 2026-05-07T14:08:32+08:00
 *
 * Unknown characters are passed through verbatim, so users may write
 * `YYYY年MM月DD日` to get `2026年05月07日`.
 */

const TOKEN_RE = /(YYYY|YY|MM|M|DD|D|HH|mm|ss)/g;

const pad2 = (n: number): string => (n < 10 ? `0${n}` : `${n}`);

export function formatDate(d: Date, fmt: string): string {
  if (fmt === 'timestamp') {
    return Math.floor(d.getTime() / 1000).toString();
  }
  if (fmt === 'iso') {
    return toLocalIso(d);
  }
  const year = d.getFullYear();
  const month = d.getMonth() + 1;
  const day = d.getDate();
  const hour = d.getHours();
  const min = d.getMinutes();
  const sec = d.getSeconds();
  return fmt.replace(TOKEN_RE, (m) => {
    switch (m) {
      case 'YYYY': return String(year);
      case 'YY':   return pad2(year % 100);
      case 'MM':   return pad2(month);
      case 'M':    return String(month);
      case 'DD':   return pad2(day);
      case 'D':    return String(day);
      case 'HH':   return pad2(hour);
      case 'mm':   return pad2(min);
      case 'ss':   return pad2(sec);
      default:     return m;
    }
  });
}

function toLocalIso(d: Date): string {
  const tz = -d.getTimezoneOffset(); // minutes east of UTC
  const sign = tz >= 0 ? '+' : '-';
  const abs = Math.abs(tz);
  const tzh = pad2(Math.floor(abs / 60));
  const tzm = pad2(abs % 60);
  const datePart = `${d.getFullYear()}-${pad2(d.getMonth() + 1)}-${pad2(d.getDate())}`;
  const timePart = `${pad2(d.getHours())}:${pad2(d.getMinutes())}:${pad2(d.getSeconds())}`;
  return `${datePart}T${timePart}${sign}${tzh}:${tzm}`;
}

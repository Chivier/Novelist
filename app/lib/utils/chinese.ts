/**
 * Chinese text utilities: Simplified/Traditional conversion and Pinyin generation.
 * All dependencies are lazy-loaded via dynamic import() to avoid bundling
 * large dictionaries upfront.
 */

// --- Simplified <-> Traditional conversion (opencc-js) ---

type Converter = (text: string) => string;

let converterS2TPromise: Promise<Converter> | null = null;
let converterT2SPromise: Promise<Converter> | null = null;

function getS2TConverter(): Promise<Converter> {
  if (!converterS2TPromise) {
    converterS2TPromise = import('opencc-js/cn2t').then(OpenCC =>
      OpenCC.Converter({ from: 'cn', to: 'tw' })
    );
  }
  return converterS2TPromise;
}

function getT2SConverter(): Promise<Converter> {
  if (!converterT2SPromise) {
    converterT2SPromise = import('opencc-js/t2cn').then(OpenCC =>
      OpenCC.Converter({ from: 'tw', to: 'cn' })
    );
  }
  return converterT2SPromise;
}

/** Convert Simplified Chinese text to Traditional Chinese. */
export async function simplifiedToTraditional(text: string): Promise<string> {
  const converter = await getS2TConverter();
  return converter(text);
}

/** Convert Traditional Chinese text to Simplified Chinese. */
export async function traditionalToSimplified(text: string): Promise<string> {
  const converter = await getT2SConverter();
  return converter(text);
}

// --- Pinyin generation (pinyin-pro) ---

/** Generate Pinyin with tone marks for the given Chinese text. */
export async function toPinyin(text: string): Promise<string> {
  const { pinyin } = await import('pinyin-pro');
  return pinyin(text, { toneType: 'symbol', type: 'string' });
}

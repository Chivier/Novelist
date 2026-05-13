/// <reference types="vite/client" />

declare module 'opencc-js' {
  interface ConverterOptions {
    from: 'cn' | 'tw' | 'hk' | 'jp' | 't';
    to: 'cn' | 'tw' | 'hk' | 'jp' | 't';
  }
  export function Converter(options: ConverterOptions): (text: string) => string;
}

declare module 'opencc-js/cn2t' {
  export function Converter(options: { from: 'cn'; to: 'tw' }): (text: string) => string;
}

declare module 'opencc-js/t2cn' {
  export function Converter(options: { from: 'tw'; to: 'cn' }): (text: string) => string;
}

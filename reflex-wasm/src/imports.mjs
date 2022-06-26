// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default function imports(wasi, getModule) {
  const I64_MAX_VALUE = BigInt('0xFFFFFFFFFFFFFFFF');
  return {
    Math: {
      remainder: (x, y) => x % y,
      acos: Math.acos,
      acosh: Math.acosh,
      asin: Math.asin,
      asinh: Math.asinh,
      atan: Math.atan,
      atan2: Math.atan2,
      atanh: Math.atanh,
      cbrt: Math.cbrt,
      cos: Math.cos,
      cosh: Math.cosh,
      exp: Math.exp,
      expm1: Math.expm1,
      hypot: Math.hypot,
      log: Math.log,
      log2: Math.log2,
      log10: Math.log10,
      log1p: Math.log1p,
      pow: Math.pow,
      sin: Math.sin,
      sinh: Math.sinh,
      sqrt: Math.sqrt,
      tan: Math.tan,
      tanh: Math.tanh,
    },
    Number: {
      toString: (value, offset) => {
        const instance = getModule();
        const valueString = (() => {
          if (isNaN(value)) {
            return 'NaN';
          } else if (value === Infinity) {
            return 'Infinity';
          } else if (value === -Infinity) {
            return '-Infinity';
          } else if (value === Math.floor(value)) {
            return value.toFixed(1);
          } else {
            return value.toString(10);
          }
        })();
        const bytes = new TextEncoder().encode(valueString);
        const length = bytes.length;
        instance.exports.allocate(length);
        new Uint8Array(instance.exports.memory.buffer, offset, length).set(bytes);
        return length;
      },
    },
    Date: {
      parse: (offset, length) => {
        const instance = getModule();
        const dateString = (() => {
          try {
            return new TextDecoder('utf-8', { fatal: true }).decode(
              new Uint8Array(instance.exports.memory.buffer, offset, length),
            );
          } catch {
            return null;
          }
        })();
        if (dateString === null) return I64_MAX_VALUE;
        const date = new Date(dateString);
        const timestamp = date.getTime();
        if (isNaN(timestamp)) return I64_MAX_VALUE;
        return BigInt(timestamp);
      },
      toISOString: (timestamp, offset) => {
        const dateString = (() => {
          try {
            return new Date(Number(timestamp)).toISOString();
          } catch {
            return null;
          }
        })();
        if (!dateString) return 0;
        const instance = getModule();
        const bytes = new TextEncoder().encode(dateString);
        const length = bytes.length;
        instance.exports.allocate(length);
        new Uint8Array(instance.exports.memory.buffer, offset, length).set(bytes);
        return length;
      },
    },
    wasi_snapshot_preview1: wasi.wasiImport,
  };
}

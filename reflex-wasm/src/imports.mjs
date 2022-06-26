// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default function imports(wasi) {
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
    wasi_snapshot_preview1: wasi.wasiImport,
  };
}

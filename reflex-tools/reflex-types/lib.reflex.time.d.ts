// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw

declare module 'reflex::time' {
  export function timeout(duration: number, token: symbol): null;
  export function now(options: { interval: number }): number;
}

// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw

declare module 'reflex::state' {
  export function get<T>(uid: symbol, defaultValue: T): T;
  export function set<T>(uid: symbol, value: T, token: symbol): T;
  export function increment(uid: symbol, token: symbol): number;
  export function decrement(uid: symbol, token: symbol): number;
  export function variable<T>(uid: symbol): [T, (value: T, token: symbol) => T];
  export function scan<T, V>(
    input: () => T,
    seed: V,
    reduce: (previous: V, item: T) => V,
  ): V;
}

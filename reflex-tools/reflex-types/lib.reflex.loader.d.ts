// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw

declare module 'reflex::loader' {
  export default class DataLoader<K, V> {
    constructor(
      name: string,
      factory: (keys: Array<K>) => Array<V> | Map<K, V>,
    );
    load(key: K): V;
    loadMany(keys: Array<K>): Array<V>;
  }
}

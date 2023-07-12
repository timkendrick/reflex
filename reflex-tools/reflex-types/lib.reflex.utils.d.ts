// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw

declare module 'reflex::utils' {
  export function graph<T>(factory: (value: T) => T): T;
  export function log<T>(value: T, ...args: any[]): T;
  export function pending(): never;
}

// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw

declare module 'reflex::invalidation' {
  export function backoff(retryIndex: number): number;
  export function poll<T>(duration: number, factory: (token: symbol) => T): T;
  export function retryErrors<T>(
    options: { delay: (retryIndex: number) => number; timeout: number },
    factory: (token: symbol) => T,
  ): T;
}

// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw

declare module 'reflex::http' {
  export class Request {
    constructor(init: {
      url: string;
      method: string;
      headers: Record<string, string>;
      body: string | null;
      token: symbol | null;
    });
    url: string;
    method: string;
    headers: Record<string, string>;
    body: string | null;
    token: symbol | null;
  }

  export interface Response<T> {
    status: number;
    ok: boolean;
    text(): string;
    json(): T;
  }

  export function fetch<T>(url: string): Response<T>;
  export function fetch<T>(request: Request): Response<T>;
}

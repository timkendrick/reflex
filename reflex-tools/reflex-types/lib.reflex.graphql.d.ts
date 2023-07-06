// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw

declare module 'reflex::graphql' {
  export interface GraphRoot {
    query: Record<string, any> | null;
    mutation: Record<string, any> | null;
    subscription: Record<string, any> | null;
  }

  export class Resolver {
    constructor(root: GraphRoot);
    constructor(factory: (requestToken: symbol) => GraphRoot);
  }
}

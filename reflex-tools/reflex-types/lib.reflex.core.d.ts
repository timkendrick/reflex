// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw

declare module 'reflex::core' {
  export function abs(value: number): number;
  export function add(left: number, right: number): number;
  export function and(left: boolean, right: () => boolean): boolean;
  export function apply<F extends (...args: A) => T, A extends Array<any>, T>(
    target: F,
    args: A,
  ): T;
  export function car<T>(pair: [T, any]): T;
  export function cdr<T>(pair: [any, T]): T;
  export function ceil(left: number, right: number): number;
  export function chain<T>(left: Array<T>, right: Array<T>): Array<T>;
  export function collectConstructor<K extends PropertyKey, V>(
    ...keys: Array<K>
  ): (...values: Array<V>) => Record<K, V>;
  export function collectHashMap<K, V>(
    ...keysAndValues: Array<K | V>
  ): Map<K, V>;
  export function collectHashSet<T>(...values: Array<T>): Set<T>;
  export function collectList<T>(...items: Array<T>): Array<T>;
  export function collectRecord<K extends PropertyKey, V>(
    ...keysAndValues: Array<K | V>
  ): Record<K, V>;
  export function collectSignal(...signals: Array<never>): never;
  export function concat(...args: Array<string>): string;
  export function cons<L, R>(left: L, right: R): [L, R];
  export function constructHashMap<K, V>(
    keys: Array<K>,
    values: Array<V>,
  ): Map<K, V>;
  export function constructHashSet<T>(...values: Array<T>): Set<T>;
  export function constructRecord<K extends PropertyKey, V>(
    keys: Array<K>,
    values: Array<V>,
  ): Record<K, V>;
  export function constructList<T>(...items: Array<T>): Array<T>;
  export function contains<K>(target: Map<K, any>, key: K): boolean;
  export function contains<T>(target: Set<T>, value: T): boolean;
  export function contains<K extends PropertyKey>(
    target: Record<K, any>,
    key: K,
  ): boolean;
  export function divide(left: number, right: number): number;
  export function effect<T, V>(type: string, payload: T, token: symbol): V;
  export function endsWith(target: string, pattern: string): boolean;
  export function eq<T>(left: T, right: T): boolean;
  export function equal(left: number, right: number): boolean;
  export function filter<T>(
    target: Array<T>,
    predicate: (item: T) => boolean,
  ): Array<T>;
  export function flatten<T>(target: Array<Array<T>>): Array<T>;
  export function floor(value: number): number;
  export function get<K, V>(target: Map<K, V>, key: K): V | null;
  export function get<
    K extends PropertyKey,
    V,
    T extends {
      K: V;
    },
  >(target: T, key: K): V | null;
  export function get<K extends PropertyKey, V>(
    target: Record<K, V>,
    key: K,
  ): V | null;
  export function get<T>(target: Array<T>, index: T): T | null;
  export function gt(left: number, right: number): boolean;
  export function gte(left: number, right: number): boolean;
  export function hash(...args: Array<any>): symbol;
  export function ifError<T, V>(
    target: () => T,
    handler: (error: Error) => V,
  ): T | V;
  export function ifPending<T, V>(target: () => T, placeholder: () => V): T | V;
  export function insert<K, V>(target: Map<K, V>, key: K, value: V): Map<K, V>;
  export function keys<K>(target: Map<K, any>): Array<K>;
  export function keys<K extends PropertyKey>(target: Record<K, any>): Array<K>;
  export function keys(target: Array<any>): Array<number>;
  export function length(target: Array<any>): number;
  export function length(target: string): number;
  export function length(target: Map<any, any>): number;
  export function length(target: Set<any>): number;
  export function lt(left: number, right: number): boolean;
  export function lte(left: number, right: number): boolean;
  export function map<T, V>(
    target: Array<T>,
    iteratee: (item: T) => V,
  ): Array<V>;
  export function max(left: number, right: number): number;
  export function merge<K extends PropertyKey, V>(
    ...args: Array<Record<K, V> | null | undefined>
  ): Record<K, V>;
  export function min(left: number, right: number): number;
  export function multiply(left: number, right: number): number;
  export function not(value: boolean): boolean;
  export function or(left: boolean, right: () => boolean): boolean;
  export function pow(base: number, exponent: number): number;
  export function push<T>(target: Array<T>, value: T): Array<T>;
  export function pushFront<T>(target: Array<T>, value: T): Array<T>;
  export function raise(payload: any): never;
  export function reduce<T, V>(
    target: Array<T>,
    iteratee: (acc: V, item: T) => V,
    seed: V,
  ): V;
  export function remainder(value: number, divisor: number): number;
  export function replace(
    target: string,
    pattern: string,
    replacement: string,
  ): string;
  export function resolveArgs<T extends (...args: Array<any>) => any>(
    target: T,
  ): T;
  export function resolveDeep<T>(target: T): T;
  export function resolveHashMap<T extends Map<any, any>>(target: T): T;
  export function resolveHashSet<T extends Set<any>>(target: T): T;
  export function resolveShallow<T>(target: T): T;
  export function resolveRecord<T extends Record<PropertyKey, any>>(
    target: T,
  ): T;
  export function resolveList<T extends Array<any>>(target: T): T;
  export function round(value: number): number;
  export function sequence<T, V>(target: T, callback: (value: T) => V): V;
  export function slice<T>(
    target: Array<T>,
    startIndex: number,
    endIndex: number,
  ): Array<T>;
  export function slice(
    target: string,
    startIndex: number,
    endIndex: number,
  ): string;
  export function split(target: string, separator: string): Array<string>;
  export function startsWith(target: string, pattern: string): boolean;
  export function subtract(left: number, right: number): number;
  export function unzip<L, R>(target: Array<[L, R]>): [Array<L>, Array<R>];
  export function values<T>(target: Map<any, T>): Array<T>;
  export function values<T>(target: Set<T>): Array<T>;
  export function values<T>(target: Record<PropertyKey, T>): Array<T>;
  export function values<T>(target: Array<T>): Array<T>;
  export function zip<L, R>(left: Array<L>, right: Array<R>): Array<[L, R]>;
}

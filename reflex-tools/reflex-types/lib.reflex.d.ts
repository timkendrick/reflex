// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw

/// <reference no-default-lib="true"/>

interface Number {}

interface Symbol {}

interface RegExp {}

interface Function {}

interface CallableFunction {}

interface NewableFunction {}

interface IArguments {
  [index: number]: any;
}

declare class String {
  constructor(value: any);
  replace(pattern: string, replacement: string): string;
  slice(startIndex: number, endIndex: number): string;
  split(target: string, separator: string): Array<string>;
  startsWith(pattern: string): boolean;
  endsWith(pattern: string): boolean;
  get length(): number;
}

declare class Array<T> {
  entries(): Array<T>;
  filter(predicate: (item: T) => boolean): Array<T>;
  join(separator: string): Array<string>;
  keys(): Array<number>;
  map<V>(iteratee: (item: T) => V): Array<V>;
  reduce<V>(iteratee: (acc: V, item: T) => V, seed: V);
  slice(startIndex: number, endIndex: number): Array<T>;
  values(): Array<T>;
  get length(): number;
}

declare class Boolean {
  constructor(value: any);
}

interface Object {
  entries<K extends PropertyKey, V>(target: Record<K, V>): Array<[K, V]>;
  fromEntries<K extends PropertyKey, V>(entries: Array<[K, V]>): Record<K, V>;
  keys<K extends PropertyKey>(target: Record<K, any>): Array<K>;
  values<V>(target: Record<PropertyKey, V>): Array<V>;
}
declare const Object: Object;

declare class Error {
  constructor(message: string);
  name: string;
  message: string;
}

declare class AggregateError extends Error {
  constructor(errors: Array<Error>);
  errors: Array<Error>;
}

interface Math {
  abs(left: number, right: number): number;
  ceil(value: number): number;
  floor(value: number): number;
  max(left: number, right: number): number;
  min(left: number, right: number): number;
  pow(base: number, exponent: number): number;
  round(value: number): number;
}
declare const Math: Math;

declare class Map<K, V> {
  constructor(entries: Array<[K, V]>);
  entries(): Array<[K, V]>;
  get(key: K): V | null;
  has(key: K): boolean;
  keys(): Array<K>;
  set(key: K, value: V): Map<K, V>;
  size(): number;
  values(): Array<V>;
}

declare class Set<T> {
  constructor(entries: Array<T>);
  add(value: T): Set<T>;
  entries(): Array<[T, T]>;
  has(value: T): boolean;
  keys(): Array<T>;
  size(): number;
  values(): Array<T>;
}

declare class Date {
  constructor(datestring: string);
  constructor(timestamp: number);
  getTime(): number;
}

type JsonValue =
  | string
  | number
  | boolean
  | JsonRecord
  | JsonArray
  | null
  | undefined;
interface JsonRecord extends Record<PropertyKey, JsonValue> {}
interface JsonArray extends Array<JsonValue> {}

interface JSON {
  parse<T>(value: string): T;
  stringify(value: JsonValue): string;
}
declare const JSON: JSON;

declare var NaN: number;
declare var Infinity: number;

declare function isFinite(value: number): boolean;

declare function encodeURIComponent(value: string): string;

declare function parseFloat(value: number): number;

declare function parseInt(value: number): number;

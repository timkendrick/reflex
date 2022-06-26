// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::Float', (test) => {
    test('format', (assert, { createFloat, format }) => {
      assert.strictEqual(format(createFloat(0.0)), '0.0');
      assert.strictEqual(format(createFloat(-0.0)), '0.0');
      assert.strictEqual(format(createFloat(3.0)), '3.0');
      assert.strictEqual(format(createFloat(-3.0)), '-3.0');
      assert.strictEqual(format(createFloat(3.142)), '3.142');
      assert.strictEqual(format(createFloat(-3.142)), '-3.142');
    });

    test('hash', (assert, { createFloat, hash }) => {
      assert.strictEqual(hash(createFloat(0)), hash(createFloat(0)));
      assert.strictEqual(hash(createFloat(3)), hash(createFloat(3)));
      assert.strictEqual(hash(createFloat(-3)), hash(createFloat(-3)));
      assert.notStrictEqual(hash(createFloat(-3)), hash(createFloat(3)));
      assert.strictEqual(hash(createFloat(3.142)), hash(createFloat(3.142)));
      assert.strictEqual(hash(createFloat(-3.142)), hash(createFloat(-3.142)));
      assert.strictEqual(hash(createFloat(NaN)), hash(createFloat(NaN)));
      assert.strictEqual(hash(createFloat(Infinity)), hash(createFloat(Infinity)));
      assert.strictEqual(hash(createFloat(-Infinity)), hash(createFloat(-Infinity)));
      assert.notStrictEqual(hash(createFloat(Infinity)), hash(createFloat(-Infinity)));
      assert.notStrictEqual(hash(createFloat(NaN)), hash(createFloat(Infinity)));
      assert.notStrictEqual(hash(createFloat(NaN)), hash(createFloat(-Infinity)));
      assert.strictEqual(
        new Set(Array.from({ length: 20 }).map((_, i) => hash(createFloat(i)))).size,
        20,
      );
    });

    test('equals', (assert, { createFloat, equals }) => {
      assert.strictEqual(equals(createFloat(0), createFloat(0)), true);
      assert.strictEqual(equals(createFloat(3), createFloat(3)), true);
      assert.strictEqual(equals(createFloat(-3), createFloat(-3)), true);
      assert.strictEqual(equals(createFloat(-3), createFloat(3)), false);
      assert.strictEqual(equals(createFloat(3.142), createFloat(3.142)), true);
      assert.strictEqual(equals(createFloat(-3.142), createFloat(-3.142)), true);
      assert.strictEqual(equals(createFloat(NaN), createFloat(NaN)), true);
      assert.strictEqual(equals(createFloat(NaN), createFloat(Infinity)), false);
      assert.strictEqual(equals(createFloat(NaN), createFloat(-Infinity)), false);
      assert.strictEqual(equals(createFloat(Infinity), createFloat(NaN)), false);
      assert.strictEqual(equals(createFloat(Infinity), createFloat(Infinity)), true);
      assert.strictEqual(equals(createFloat(Infinity), createFloat(-Infinity)), false);
      assert.strictEqual(equals(createFloat(-Infinity), createFloat(NaN)), false);
      assert.strictEqual(equals(createFloat(-Infinity), createFloat(Infinity)), false);
      assert.strictEqual(equals(createFloat(-Infinity), createFloat(-Infinity)), true);
    });
  });
};

// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::String', (test) => {
    test('format', (assert, { createString, format }) => {
      assert.strictEqual(format(createString('')), '""');
      assert.strictEqual(format(createString('foo')), '"foo"');
      assert.strictEqual(format(createString('"')), '"\\""');
    });

    test('hash', (assert, { createString, hash }) => {
      assert.strictEqual(hash(createString('')), hash(createString('')));
      assert.strictEqual(hash(createString('foo')), hash(createString('foo')));
      assert.notStrictEqual(hash(createString('foo')), hash(createString('bar')));
      assert.strictEqual(
        new Set(
          Array.from({ length: 20 }).map((_, i) =>
            hash(createString(String.fromCharCode('a'.charCodeAt(0) + i))),
          ),
        ).size,
        20,
      );
    });

    test('equals', (assert, { createString, equals }) => {
      assert.strictEqual(equals(createString(''), createString('')), true);
      assert.strictEqual(equals(createString('foo'), createString('foo')), true);
      assert.strictEqual(equals(createString(''), createString('foo')), false);
      assert.strictEqual(equals(createString('foo'), createString('')), false);
      assert.strictEqual(equals(createString('foo'), createString('bar')), false);
      assert.strictEqual(equals(createString('foo'), createString('fooo')), false);
      assert.strictEqual(equals(createString('foo'), createString('fo')), false);
    });

    test('toJson', (assert, { createString, getStringValue, toJson }) => {
      assert.strictEqual(getStringValue(toJson(createString(''))), '""');
      assert.strictEqual(getStringValue(toJson(createString('foo'))), '"foo"');
      assert.strictEqual(getStringValue(toJson(createString('\b'))), '"\\b"');
      assert.strictEqual(getStringValue(toJson(createString('\f'))), '"\\f"');
      assert.strictEqual(getStringValue(toJson(createString('\n'))), '"\\n"');
      assert.strictEqual(getStringValue(toJson(createString('\r'))), '"\\r"');
      assert.strictEqual(getStringValue(toJson(createString('\t'))), '"\\t"');
      assert.strictEqual(getStringValue(toJson(createString('"'))), '"\\\""');
      assert.strictEqual(getStringValue(toJson(createString('\\'))), '"\\\\"');
      assert.strictEqual(getStringValue(toJson(createString('""'))), '"\\\"\\\""');
      assert.strictEqual(getStringValue(toJson(createString('"foo"'))), '"\\\"foo\\\""');
      assert.strictEqual(getStringValue(toJson(createString('foo "bar" baz'))), '"foo \\\"bar\\\" baz"');
    });
  });
};

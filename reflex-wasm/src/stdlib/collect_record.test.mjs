// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_CollectRecord', (test) => {
    test('()', (assert, {
      createApplication,
      createEmptyList,
      createBuiltin,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(createBuiltin(Stdlib.CollectRecord), createEmptyList());
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), '{}');
      assert.strictEqual(format(dependencies), 'NULL');
    });

    test('(String, Int, String, Int, String, Int)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
      createInt,
      createLambda,
      createList,
      createString,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectRecord),
          createList([
            createString('foo'),
            createInt(3),
            createString('bar'),
            createInt(4),
            createString('baz'),
            createInt(5),
          ]),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{ "foo": 3, "bar": 4, "baz": 5 }');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectRecord),
          createList([
            createApplication(createLambda(0, createString('foo')), createEmptyList()),
            createApplication(createLambda(0, createInt(3)), createEmptyList()),
            createApplication(createLambda(0, createString('bar')), createEmptyList()),
            createApplication(createLambda(0, createInt(4)), createEmptyList()),
            createApplication(createLambda(0, createString('baz')), createEmptyList()),
            createApplication(createLambda(0, createInt(5)), createEmptyList()),
          ]),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{ "foo": 3, "bar": 4, "baz": 5 }');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('Duplicate keys', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createList,
      createString,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectRecord),
          createList([
            createString('foo'),
            createInt(3),
            createString('bar'),
            createInt(4),
            createString('foo'),
            createInt(5),
          ]),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{ "foo": 3, "bar": 4, "foo": 5 }');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('Trailing keys', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createList,
      createString,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectRecord),
          createList([
            createString('foo'),
            createInt(3),
            createString('bar'),
            createInt(4),
            createString('baz'),
          ]),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{ "foo": 3, "bar": 4 }');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};

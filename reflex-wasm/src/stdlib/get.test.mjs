// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Get', (test) => {
    test('(List, Int)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createPair,
      createTriple,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const list = createTriple(createInt(3), createInt(4), createInt(5));
        const expression = createApplication(
          createBuiltin(Stdlib.Get),
          createPair(list, createInt(0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const list = createTriple(createInt(3), createInt(4), createInt(5));
        const expression = createApplication(
          createBuiltin(Stdlib.Get),
          createPair(list, createInt(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{<InvalidFunctionArgsCondition:Get([3, 4, 5], 3)>}');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Record, String)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createPair,
      createRecord,
      createString,
      createTriple,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const record = createRecord(
          createTriple(createString('foo'), createString('bar'), createString('baz')),
          createTriple(createInt(3), createInt(4), createInt(5)),
        );
        const expression = createApplication(
          createBuiltin(Stdlib.Get),
          createPair(record, createString('foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const record = createRecord(
          createTriple(createString('foo'), createString('bar'), createString('baz')),
          createTriple(createInt(3), createInt(4), createInt(5)),
        );
        const expression = createApplication(
          createBuiltin(Stdlib.Get),
          createPair(record, createString('invalid')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<InvalidFunctionArgsCondition:Get({ "foo": 3, "bar": 4, "baz": 5 }, "invalid")>}',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Record, Symbol)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createPair,
      createRecord,
      createSymbol,
      createTriple,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      const record = createRecord(
        createTriple(createSymbol(123), createSymbol(456), createSymbol(789)),
        createTriple(createInt(3), createInt(4), createInt(5)),
      );
      const expression = createApplication(
        createBuiltin(Stdlib.Get),
        createPair(record, createSymbol(123)),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), '3');
      assert.deepEqual(getStateDependencies(dependencies), []);
    });

    test('(Hashmap, String)', (assert, {
      createApplication,
      createBuiltin,
      createHashmap,
      createInt,
      createPair,
      createString,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      const hashmap = createHashmap([
        [createString('foo'), createInt(3)],
        [createString('bar'), createInt(4)],
        [createString('baz'), createInt(5)],
      ]);
      const expression = createApplication(
        createBuiltin(Stdlib.Get),
        createPair(hashmap, createString('foo')),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), '3');
      assert.deepEqual(getStateDependencies(dependencies), []);
    });

    test('(Hashmap, Symbol)', (assert, {
      createApplication,
      createBuiltin,
      createHashmap,
      createInt,
      createPair,
      createSymbol,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      const hashmap = createHashmap([
        [createSymbol(123), createInt(3)],
        [createSymbol(456), createInt(4)],
        [createSymbol(789), createInt(5)],
      ]);
      const expression = createApplication(
        createBuiltin(Stdlib.Get),
        createPair(hashmap, createSymbol(123)),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), '3');
      assert.deepEqual(getStateDependencies(dependencies), []);
    });
  });
};

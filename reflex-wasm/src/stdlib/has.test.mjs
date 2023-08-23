// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Has', (test) => {
    test('(List, Int)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
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
        const list = createEmptyList();
        const expression = createApplication(
          createBuiltin(Stdlib.Has),
          createPair(list, createInt(0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const list = createTriple(createInt(3), createInt(4), createInt(5));
        const expression = createApplication(
          createBuiltin(Stdlib.Has),
          createPair(list, createInt(0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const list = createTriple(createInt(3), createInt(4), createInt(5));
        const expression = createApplication(
          createBuiltin(Stdlib.Has),
          createPair(list, createInt(1)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const list = createTriple(createInt(3), createInt(4), createInt(5));
        const expression = createApplication(
          createBuiltin(Stdlib.Has),
          createPair(list, createInt(2)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const list = createTriple(createInt(3), createInt(4), createInt(5));
        const expression = createApplication(
          createBuiltin(Stdlib.Has),
          createPair(list, createInt(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
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
          createBuiltin(Stdlib.Has),
          createPair(record, createString('foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const record = createRecord(
          createTriple(createString('foo'), createString('bar'), createString('baz')),
          createTriple(createInt(3), createInt(4), createInt(5)),
        );
        const expression = createApplication(
          createBuiltin(Stdlib.Has),
          createPair(record, createString('qux')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
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
      (() => {
        const record = createRecord(
          createTriple(createSymbol(123), createSymbol(456), createSymbol(789)),
          createTriple(createInt(3), createInt(4), createInt(5)),
        );
        const expression = createApplication(
          createBuiltin(Stdlib.Has),
          createPair(record, createSymbol(123)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const record = createRecord(
          createTriple(createSymbol(123), createSymbol(456), createSymbol(789)),
          createTriple(createInt(3), createInt(4), createInt(5)),
        );
        const expression = createApplication(
          createBuiltin(Stdlib.Has),
          createPair(record, createSymbol(999)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
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
      (() => {
        const hashmap = createHashmap([
          [createString('foo'), createInt(3)],
          [createString('bar'), createInt(4)],
          [createString('baz'), createInt(5)],
        ]);
        const expression = createApplication(
          createBuiltin(Stdlib.Has),
          createPair(hashmap, createString('foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const hashmap = createHashmap([
          [createString('foo'), createInt(3)],
          [createString('bar'), createInt(4)],
          [createString('baz'), createInt(5)],
        ]);
        const expression = createApplication(
          createBuiltin(Stdlib.Has),
          createPair(hashmap, createString('qux')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
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
      (() => {
        const hashmap = createHashmap([
          [createSymbol(123), createInt(3)],
          [createSymbol(456), createInt(4)],
          [createSymbol(789), createInt(5)],
        ]);
        const expression = createApplication(
          createBuiltin(Stdlib.Has),
          createPair(hashmap, createSymbol(123)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const hashmap = createHashmap([
          [createSymbol(123), createInt(3)],
          [createSymbol(456), createInt(4)],
          [createSymbol(789), createInt(5)],
        ]);
        const expression = createApplication(
          createBuiltin(Stdlib.Has),
          createPair(hashmap, createSymbol(999)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Hashset, String)', (assert, {
      createApplication,
      createBuiltin,
      createHashset,
      createInt,
      createPair,
      createString,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const hashset = createHashset([
          createString('foo'),
          createString('bar'),
          createString('baz'),
        ]);
        const expression = createApplication(
          createBuiltin(Stdlib.Has),
          createPair(hashset, createString('foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const hashmap = createHashset([
          createString('foo'),
          createString('bar'),
          createString('baz'),
        ]);
        const expression = createApplication(
          createBuiltin(Stdlib.Has),
          createPair(hashmap, createString('qux')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Hashset, Symbol)', (assert, {
      createApplication,
      createBuiltin,
      createHashset,
      createPair,
      createSymbol,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const hashmap = createHashset([createSymbol(123), createSymbol(456), createSymbol(789)]);
        const expression = createApplication(
          createBuiltin(Stdlib.Has),
          createPair(hashmap, createSymbol(123)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const hashmap = createHashset([createSymbol(123), createSymbol(456), createSymbol(789)]);
        const expression = createApplication(
          createBuiltin(Stdlib.Has),
          createPair(hashmap, createSymbol(999)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};

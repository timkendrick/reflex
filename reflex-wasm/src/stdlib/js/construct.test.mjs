// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Construct', (test) => {
    test('(Constructor, Record)', (assert, {
      createApplication,
      createBuiltin,
      createConstructor,
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
        const expression = createApplication(
          createBuiltin(Stdlib.Construct),
          createPair(
            createConstructor(
              createTriple(createString('foo'), createString('bar'), createString('baz')),
            ),
            createRecord(
              createTriple(createString('foo'), createString('bar'), createString('baz')),
              createTriple(createInt(3), createInt(4), createInt(5)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{ "foo": 3, "bar": 4, "baz": 5 }');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Construct),
          createPair(
            createConstructor(
              createTriple(createString('foo'), createString('bar'), createString('baz')),
            ),
            createRecord(
              createTriple(createString('baz'), createString('bar'), createString('foo')),
              createTriple(createInt(5), createInt(4), createInt(3)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{ "foo": 3, "bar": 4, "baz": 5 }');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Constructor, List)', (assert, {
      createApplication,
      createBuiltin,
      createConstructor,
      createInt,
      createPair,
      createString,
      createTriple,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Construct),
          createPair(
            createConstructor(
              createTriple(createString('foo'), createString('bar'), createString('baz')),
            ),
            createTriple(createInt(3), createInt(4), createInt(5)),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{ "foo": 3, "bar": 4, "baz": 5 }');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Constructor, Iterator)', (assert, {
      createApplication,
      createBuiltin,
      createConstructor,
      createPair,
      createRangeIterator,
      createString,
      createTriple,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Construct),
          createPair(
            createConstructor(
              createTriple(createString('foo'), createString('bar'), createString('baz')),
            ),
            createRangeIterator(3, 3),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{ "foo": 3, "bar": 4, "baz": 5 }');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Builtin, Int, Int)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createTriple,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Construct),
          createTriple(createBuiltin(Stdlib.Subtract), createInt(3), createInt(4)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3 - 4}`);
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};

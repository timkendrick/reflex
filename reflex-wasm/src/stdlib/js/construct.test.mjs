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
        assert.strictEqual(format(dependencies), 'NULL');
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
        assert.strictEqual(format(dependencies), 'NULL');
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
        assert.strictEqual(format(dependencies), 'NULL');
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
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Builtin, List)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createPair,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Construct),
          createPair(createBuiltin(Stdlib.Subtract), createPair(createInt(3), createInt(4))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3 - 4}`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Builtin, Iterator)', (assert, {
      createApplication,
      createBuiltin,
      createPair,
      createRangeIterator,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Construct),
          createPair(createBuiltin(Stdlib.Subtract), createRangeIterator(3, 2)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${3 - 4}`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};

// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::Constructor', (test) => {
    test('format', (assert, {
      createConstructor,
      createEmptyList,
      createString,
      createTriple,
      createUnitList,
      format,
    }) => {
      assert.strictEqual(format(createConstructor(createEmptyList())), 'Constructor({})');
      assert.strictEqual(
        format(createConstructor(createUnitList(createString('foo')))),
        'Constructor({"foo"})',
      );
      assert.strictEqual(
        format(
          createConstructor(
            createTriple(createString('foo'), createString('bar'), createString('baz')),
          ),
        ),
        'Constructor({"foo", "bar", "baz"})',
      );
    });

    test('hash', (assert, {
      createConstructor,
      createEmptyList,
      createString,
      createTriple,
      createUnitList,
      hash,
    }) => {
      assert.strictEqual(
        hash(createConstructor(createEmptyList())),
        hash(createConstructor(createEmptyList())),
      );
      assert.notStrictEqual(
        hash(createConstructor(createEmptyList())),
        hash(createConstructor(createUnitList(createString('foo')))),
      );
      assert.strictEqual(
        hash(createConstructor(createUnitList(createString('foo')))),
        hash(createConstructor(createUnitList(createString('foo')))),
      );
      assert.notStrictEqual(
        hash(createConstructor(createUnitList(createString('foo')))),
        hash(createConstructor(createUnitList(createString('bar')))),
      );
      assert.strictEqual(
        hash(
          createConstructor(
            createTriple(createString('foo'), createString('bar'), createString('baz')),
          ),
        ),
        hash(
          createConstructor(
            createTriple(createString('foo'), createString('bar'), createString('baz')),
          ),
        ),
      );
      assert.notStrictEqual(
        hash(
          createConstructor(
            createTriple(createString('foo'), createString('bar'), createString('baz')),
          ),
        ),
        hash(
          createConstructor(
            createTriple(createString('foo'), createString('bar'), createString('qux')),
          ),
        ),
      );
      assert.notStrictEqual(
        hash(createConstructor(createUnitList(createString('foo')))),
        hash(
          createConstructor(
            createTriple(createString('foo'), createString('bar'), createString('qux')),
          ),
        ),
      );
    });

    test('equals', (assert, {
      createConstructor,
      createEmptyList,
      createString,
      createTriple,
      createUnitList,
      equals,
    }) => {
      assert.strictEqual(
        equals(createConstructor(createEmptyList()), createConstructor(createEmptyList())),
        true,
      );
      assert.strictEqual(
        equals(
          createConstructor(createEmptyList()),
          createConstructor(createUnitList(createString('foo'))),
        ),
        false,
      );
      assert.strictEqual(
        equals(
          createConstructor(createUnitList(createString('foo'))),
          createConstructor(createUnitList(createString('foo'))),
        ),
        true,
      );
      assert.strictEqual(
        equals(
          createConstructor(createUnitList(createString('foo'))),
          createConstructor(createUnitList(createString('bar'))),
        ),
        false,
      );
      assert.strictEqual(
        equals(
          createConstructor(
            createTriple(createString('foo'), createString('bar'), createString('baz')),
          ),
          createConstructor(
            createTriple(createString('foo'), createString('bar'), createString('baz')),
          ),
        ),
        true,
      );
      assert.strictEqual(
        equals(
          createConstructor(
            createTriple(createString('foo'), createString('bar'), createString('baz')),
          ),
          createConstructor(
            createTriple(createString('foo'), createString('bar'), createString('qux')),
          ),
        ),
        false,
      );
      assert.strictEqual(
        equals(
          createConstructor(createUnitList(createString('foo'))),
          createConstructor(
            createTriple(createString('foo'), createString('bar'), createString('qux')),
          ),
        ),
        false,
      );
    });

    test('arity', (assert, {
      createConstructor,
      createEmptyList,
      createPair,
      createString,
      createTriple,
      createUnitList,
      arity,
      FALSE,
    }) => {
      assert.deepEqual(arity(createConstructor(createEmptyList())), [0, FALSE]);
      assert.deepEqual(arity(createConstructor(createUnitList(createString('foo')))), [1, FALSE]);
      assert.deepEqual(
        arity(createConstructor(createPair(createString('foo'), createString('bar')))),
        [2, FALSE],
      );
      assert.deepEqual(
        arity(
          createConstructor(
            createTriple(createString('foo'), createString('bar'), createString('baz')),
          ),
        ),
        [3, FALSE],
      );
    });

    test('constructor application', (assert, {
      createApplication,
      createConstructor,
      createEmptyList,
      createInt,
      createString,
      createTriple,
      evaluate,
      format,
      NULL,
    }) => {
      (function () {
        const expression = createApplication(
          createConstructor(createEmptyList()),
          createEmptyList(),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{}');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (function () {
        const expression = createApplication(
          createConstructor(
            createTriple(createString('foo'), createString('bar'), createString('baz')),
          ),
          createTriple(createInt(3), createInt(4), createInt(5)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{ "foo": 3, "bar": 4, "baz": 5 }');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('invalid constructor application', (assert, {
      createApplication,
      createConstructor,
      createEmptyList,
      createInt,
      createString,
      createTriple,
      evaluate,
      format,
      NULL,
    }) => {
      (function () {
        const expression = createApplication(
          createConstructor(createEmptyList()),
          createTriple(createInt(3), createInt(4), createInt(5)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<InvalidFunctionArgsCondition:Constructor({})(3, 4, 5)>}',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (function () {
        const expression = createApplication(
          createConstructor(
            createTriple(createString('foo'), createString('bar'), createString('baz')),
          ),
          createEmptyList(),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<InvalidFunctionArgsCondition:Constructor({"foo", "bar", "baz"})()>}',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};

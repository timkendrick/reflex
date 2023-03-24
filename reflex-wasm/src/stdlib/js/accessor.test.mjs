// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Accessor', (test) => {
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
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(
            createRecord(
              createTriple(createString('foo'), createString('bar'), createString('baz')),
              createTriple(createInt(3), createInt(4), createInt(5)),
            ),
            createString('foo'),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(
            createRecord(
              createTriple(createString('foo'), createString('bar'), createString('baz')),
              createTriple(createInt(3), createInt(4), createInt(5)),
            ),
            createString('bar'),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '4');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(
            createRecord(
              createTriple(createString('foo'), createString('bar'), createString('baz')),
              createTriple(createInt(3), createInt(4), createInt(5)),
            ),
            createString('baz'),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '5');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(
            createRecord(
              createTriple(createString('foo'), createString('bar'), createString('baz')),
              createTriple(createInt(3), createInt(4), createInt(5)),
            ),
            createString('qux'),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<InvalidFunctionArgsCondition:Accessor({ "foo": 3, "bar": 4, "baz": 5 }, "qux")>}',
        );
        assert.strictEqual(format(dependencies), 'NULL');
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
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(
            createRecord(
              createTriple(createSymbol(123), createSymbol(456), createSymbol(789)),
              createTriple(createInt(3), createInt(4), createInt(5)),
            ),
            createSymbol(123),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(
            createRecord(
              createTriple(createSymbol(123), createSymbol(456), createSymbol(789)),
              createTriple(createInt(3), createInt(4), createInt(5)),
            ),
            createSymbol(456),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '4');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(
            createRecord(
              createTriple(createSymbol(123), createSymbol(456), createSymbol(789)),
              createTriple(createInt(3), createInt(4), createInt(5)),
            ),
            createSymbol(789),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '5');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(
            createRecord(
              createTriple(createSymbol(123), createSymbol(456), createSymbol(789)),
              createTriple(createInt(3), createInt(4), createInt(5)),
            ),
            createSymbol(345),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<InvalidFunctionArgsCondition:Accessor({ Symbol(123): 3, Symbol(456): 4, Symbol(789): 5 }, Symbol(345))>}',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(List, Int)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createPair,
      createTriple,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createTriple(createInt(3), createInt(4), createInt(5)), createInt(0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createTriple(createInt(3), createInt(4), createInt(5)), createInt(1)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '4');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createTriple(createInt(3), createInt(4), createInt(5)), createInt(2)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '5');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createTriple(createInt(3), createInt(4), createInt(5)), createInt(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'null');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(List, Float)', (assert, {
      createApplication,
      createBuiltin,
      createFloat,
      createInt,
      createPair,
      createTriple,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createTriple(createInt(3), createInt(4), createInt(5)), createFloat(0.0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createTriple(createInt(3), createInt(4), createInt(5)), createFloat(1.0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '4');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createTriple(createInt(3), createInt(4), createInt(5)), createFloat(2.0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '5');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createTriple(createInt(3), createInt(4), createInt(5)), createFloat(3.0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'null');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createTriple(createInt(3), createInt(4), createInt(5)), createFloat(3.142)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<InvalidFunctionArgsCondition:Accessor([3, 4, 5], 3.142)>}',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Iterator, Int)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createPair,
      createRangeIterator,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createRangeIterator(3, 3), createInt(0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createRangeIterator(3, 3), createInt(1)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '4');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createRangeIterator(3, 3), createInt(2)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '5');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createRangeIterator(3, 3), createInt(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'null');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Iterator, Float)', (assert, {
      createApplication,
      createBuiltin,
      createFloat,
      createPair,
      createRangeIterator,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createRangeIterator(3, 3), createFloat(0.0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createRangeIterator(3, 3), createFloat(1.0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '4');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createRangeIterator(3, 3), createFloat(2.0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '5');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createRangeIterator(3, 3), createFloat(3.0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'null');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createRangeIterator(3, 3), createFloat(3.142)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<InvalidFunctionArgsCondition:Accessor(RangeIterator, 3.142)>}',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(String, Int)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createPair,
      createString,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createString('foo'), createInt(0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"f"');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createString('foo'), createInt(1)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"o"');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createString('foo'), createInt(2)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"o"');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createString('foo'), createInt(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'null');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(String, Float)', (assert, {
      createApplication,
      createBuiltin,
      createFloat,
      createPair,
      createString,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createString('foo'), createFloat(0.0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"f"');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createString('foo'), createFloat(1.0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"o"');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createString('foo'), createFloat(2.0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"o"');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createString('foo'), createFloat(3.0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'null');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createString('foo'), createFloat(3.142)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<InvalidFunctionArgsCondition:Accessor("foo", 3.142)>}',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(String, String)', (assert, {
      createApplication,
      createBuiltin,
      createPair,
      createString,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createString('foo'), createString('length')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createApplication(
            createBuiltin(Stdlib.Accessor),
            createPair(createString('foo'), createString('replace')),
          ),
          createPair(createString('f'), createString('b')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"boo"');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createApplication(
            createBuiltin(Stdlib.Accessor),
            createPair(createString('foo'), createString('split')),
          ),
          createUnitList(createString('o')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '["f", "", ""]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createString('foo'), createString('invalid')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<InvalidFunctionArgsCondition:Accessor("foo", "invalid")>}',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Hashmap, String)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
      createHashmap,
      createInt,
      createPair,
      createString,
      createUnitList,
      evaluate,
      format,
      getListItems,
      isList,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(
            createHashmap([
              [createString('foo'), createInt(3)],
              [createString('bar'), createInt(4)],
              [createString('baz'), createInt(5)],
            ]),
            createString('size'),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createApplication(
                createBuiltin(Stdlib.Accessor),
                createPair(
                  createHashmap([
                    [createString('foo'), createInt(3)],
                    [createString('bar'), createInt(4)],
                    [createString('baz'), createInt(5)],
                  ]),
                  createString('entries'),
                ),
              ),
              createEmptyList(),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isList(result));
        assert.strictEqual(
          `[${getListItems(result).map(format).sort().join(', ')}]`,
          '[["bar", 4], ["baz", 5], ["foo", 3]]',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createApplication(
            createBuiltin(Stdlib.Accessor),
            createPair(
              createHashmap([
                [createString('foo'), createInt(3)],
                [createString('bar'), createInt(4)],
                [createString('baz'), createInt(5)],
              ]),
              createString('get'),
            ),
          ),
          createUnitList(createString('foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createApplication(
            createBuiltin(Stdlib.Accessor),
            createPair(
              createHashmap([
                [createString('foo'), createInt(3)],
                [createString('bar'), createInt(4)],
                [createString('baz'), createInt(5)],
              ]),
              createString('get'),
            ),
          ),
          createUnitList(createString('bar')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '4');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createApplication(
            createBuiltin(Stdlib.Accessor),
            createPair(
              createHashmap([
                [createString('foo'), createInt(3)],
                [createString('bar'), createInt(4)],
                [createString('baz'), createInt(5)],
              ]),
              createString('has'),
            ),
          ),
          createUnitList(createString('foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createApplication(
            createBuiltin(Stdlib.Accessor),
            createPair(
              createHashmap([
                [createString('foo'), createInt(3)],
                [createString('bar'), createInt(4)],
                [createString('baz'), createInt(5)],
              ]),
              createString('has'),
            ),
          ),
          createUnitList(createString('qux')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createApplication(
                createBuiltin(Stdlib.Accessor),
                createPair(
                  createHashmap([
                    [createString('foo'), createInt(3)],
                    [createString('bar'), createInt(4)],
                    [createString('baz'), createInt(5)],
                  ]),
                  createString('keys'),
                ),
              ),
              createEmptyList(),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isList(result));
        assert.strictEqual(
          `[${getListItems(result).map(format).sort().join(', ')}]`,
          '["bar", "baz", "foo"]',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createApplication(
                createBuiltin(Stdlib.Accessor),
                createPair(
                  createHashmap([
                    [createString('foo'), createInt(3)],
                    [createString('bar'), createInt(4)],
                    [createString('baz'), createInt(5)],
                  ]),
                  createString('set'),
                ),
              ),
              createPair(createString('bar'), createInt(6)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isList(result));
        assert.strictEqual(
          `[${getListItems(result).map(format).sort().join(', ')}]`,
          '[["bar", 6], ["baz", 5], ["foo", 3]]',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createApplication(
                createBuiltin(Stdlib.Accessor),
                createPair(
                  createHashmap([
                    [createString('foo'), createInt(3)],
                    [createString('bar'), createInt(4)],
                    [createString('baz'), createInt(5)],
                  ]),
                  createString('values'),
                ),
              ),
              createEmptyList(),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isList(result));
        assert.strictEqual(`[${getListItems(result).map(format).sort().join(', ')}]`, '[3, 4, 5]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(
            createHashmap([
              [createString('foo'), createInt(3)],
              [createString('bar'), createInt(4)],
              [createString('baz'), createInt(5)],
            ]),
            createString('invalid'),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<InvalidFunctionArgsCondition:Accessor(Map(3), "invalid")>}',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Hashset, String)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
      createHashset,
      createPair,
      createString,
      createUnitList,
      evaluate,
      format,
      getListItems,
      isList,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(
            createHashset([createString('foo'), createString('bar'), createString('baz')]),
            createString('size'),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createApplication(
                createBuiltin(Stdlib.Accessor),
                createPair(
                  createHashset([createString('foo'), createString('bar'), createString('baz')]),
                  createString('add'),
                ),
              ),
              createUnitList(createString('qux')),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isList(result));
        assert.strictEqual(
          `[${getListItems(result).map(format).sort().join(', ')}]`,
          '["bar", "baz", "foo", "qux"]',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createApplication(
                createBuiltin(Stdlib.Accessor),
                createPair(
                  createHashset([createString('foo'), createString('bar'), createString('baz')]),
                  createString('entries'),
                ),
              ),
              createEmptyList(),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isList(result));
        assert.strictEqual(
          `[${getListItems(result).map(format).sort().join(', ')}]`,
          '["bar", "baz", "foo"]',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createApplication(
            createBuiltin(Stdlib.Accessor),
            createPair(
              createHashset([createString('foo'), createString('bar'), createString('baz')]),
              createString('has'),
            ),
          ),
          createUnitList(createString('foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createApplication(
            createBuiltin(Stdlib.Accessor),
            createPair(
              createHashset([createString('foo'), createString('bar'), createString('baz')]),
              createString('has'),
            ),
          ),
          createUnitList(createString('qux')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createApplication(
                createBuiltin(Stdlib.Accessor),
                createPair(
                  createHashset([createString('foo'), createString('bar'), createString('baz')]),
                  createString('values'),
                ),
              ),
              createEmptyList(),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isList(result));
        assert.strictEqual(
          `[${getListItems(result).map(format).sort().join(', ')}]`,
          '["bar", "baz", "foo"]',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(
            createHashset([createString('foo'), createString('bar'), createString('baz')]),
            createString('invalid'),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<InvalidFunctionArgsCondition:Accessor(Set(3), "invalid")>}',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(List, String)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
      createInt,
      createLambda,
      createPair,
      createString,
      createTriple,
      createUnitList,
      createVariable,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createApplication(
                createBuiltin(Stdlib.Accessor),
                createPair(
                  createTriple(createInt(3), createInt(4), createInt(5)),
                  createString('entries'),
                ),
              ),
              createEmptyList(),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createApplication(
                createBuiltin(Stdlib.Accessor),
                createPair(
                  createTriple(createInt(3), createInt(4), createInt(5)),
                  createString('filter'),
                ),
              ),
              createUnitList(
                createLambda(
                  1,
                  createApplication(
                    createBuiltin(Stdlib.Equal),
                    createPair(
                      createApplication(
                        createBuiltin(Stdlib.Remainder),
                        createPair(createVariable(0), createInt(2)),
                      ),
                      createInt(1),
                    ),
                  ),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 5]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createApplication(
                createBuiltin(Stdlib.Accessor),
                createPair(
                  createTriple(createInt(3), createInt(4), createInt(5)),
                  createString('keys'),
                ),
              ),
              createEmptyList(),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[0, 1, 2]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createEmptyList(), createString('length')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '0');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(
            createTriple(createInt(3), createInt(4), createInt(5)),
            createString('length'),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createApplication(
                createBuiltin(Stdlib.Accessor),
                createPair(
                  createTriple(createInt(3), createInt(4), createInt(5)),
                  createString('map'),
                ),
              ),
              createUnitList(
                createLambda(
                  1,
                  createApplication(
                    createBuiltin(Stdlib.Multiply),
                    createPair(createVariable(0), createInt(2)),
                  ),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `[${3 * 2}, ${4 * 2}, ${5 * 2}]`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createApplication(
                createBuiltin(Stdlib.Accessor),
                createPair(
                  createTriple(createInt(3), createInt(4), createInt(5)),
                  createString('push'),
                ),
              ),
              createUnitList(createInt(6)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `[3, 4, 5, 6]`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createApplication(
            createBuiltin(Stdlib.Accessor),
            createPair(
              createTriple(createInt(3), createInt(4), createInt(5)),
              createString('reduce'),
            ),
          ),
          createPair(createBuiltin(Stdlib.Subtract), createInt(6)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${6 - 3 - 4 - 5}`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createApplication(
                createBuiltin(Stdlib.Accessor),
                createPair(
                  createTriple(createInt(3), createInt(4), createInt(5)),
                  createString('slice'),
                ),
              ),
              createPair(createInt(1), createInt(2)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `[4, 5]`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createApplication(
                createBuiltin(Stdlib.Accessor),
                createPair(
                  createTriple(createInt(3), createInt(4), createInt(5)),
                  createString('unshift'),
                ),
              ),
              createUnitList(createInt(6)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `[6, 3, 4, 5]`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(
            createTriple(createInt(3), createInt(4), createInt(5)),
            createString('invalid'),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<InvalidFunctionArgsCondition:Accessor([3, 4, 5], "invalid")>}',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Iterator, String)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyIterator,
      createEmptyList,
      createInt,
      createLambda,
      createPair,
      createRangeIterator,
      createString,
      createUnitList,
      createVariable,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createApplication(
                createBuiltin(Stdlib.Accessor),
                createPair(createRangeIterator(3, 3), createString('entries')),
              ),
              createEmptyList(),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createApplication(
                createBuiltin(Stdlib.Accessor),
                createPair(createRangeIterator(3, 3), createString('filter')),
              ),
              createUnitList(
                createLambda(
                  1,
                  createApplication(
                    createBuiltin(Stdlib.Equal),
                    createPair(
                      createApplication(
                        createBuiltin(Stdlib.Remainder),
                        createPair(createVariable(0), createInt(2)),
                      ),
                      createInt(1),
                    ),
                  ),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 5]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createApplication(
                createBuiltin(Stdlib.Accessor),
                createPair(createRangeIterator(3, 3), createString('keys')),
              ),
              createEmptyList(),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[0, 1, 2]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createEmptyIterator(), createString('length')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '0');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createRangeIterator(3, 3), createString('length')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createApplication(
                createBuiltin(Stdlib.Accessor),
                createPair(createRangeIterator(3, 3), createString('map')),
              ),
              createUnitList(
                createLambda(
                  1,
                  createApplication(
                    createBuiltin(Stdlib.Multiply),
                    createPair(createVariable(0), createInt(2)),
                  ),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `[${3 * 2}, ${4 * 2}, ${5 * 2}]`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createApplication(
                createBuiltin(Stdlib.Accessor),
                createPair(createRangeIterator(3, 3), createString('push')),
              ),
              createUnitList(createInt(6)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `[3, 4, 5, 6]`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createApplication(
            createBuiltin(Stdlib.Accessor),
            createPair(createRangeIterator(3, 3), createString('reduce')),
          ),
          createPair(createBuiltin(Stdlib.Subtract), createInt(6)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `${6 - 3 - 4 - 5}`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createApplication(
                createBuiltin(Stdlib.Accessor),
                createPair(createRangeIterator(3, 3), createString('slice')),
              ),
              createPair(createInt(1), createInt(2)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `[4, 5]`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createApplication(
                createBuiltin(Stdlib.Accessor),
                createPair(createRangeIterator(3, 3), createString('unshift')),
              ),
              createUnitList(createInt(6)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), `[6, 3, 4, 5]`);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Accessor),
          createPair(createRangeIterator(3, 3), createString('invalid')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<InvalidFunctionArgsCondition:Accessor(RangeIterator, "invalid")>}',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};

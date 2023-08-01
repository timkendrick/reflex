// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_ResolveDeep', (test) => {
    test('(Nil)', (assert, {
      createApplication,
      createBuiltin,
      createNil,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      const expression = createApplication(
        createBuiltin(Stdlib.ResolveDeep),
        createUnitList(createNil()),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(format(result), 'null');
      assert.strictEqual(format(dependencies), 'NULL');
    });

    test('(Boolean)', (assert, {
      createApplication,
      createBuiltin,
      createBoolean,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(createBoolean(false)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(createBoolean(true)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Int)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(createInt(0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '0');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(createInt(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(createInt(-3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '-3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(List)', (assert, {
      createApplication,
      createEmptyList,
      createBuiltin,
      createInt,
      createTriple,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(createEmptyList()),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(createTriple(createInt(3), createInt(4), createInt(5))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(
            createTriple(
              createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-3))),
              createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-4))),
              createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-5))),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(
            createTriple(
              createApplication(
                createBuiltin(Stdlib.Identity),
                createUnitList(
                  createTriple(
                    createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-1))),
                    createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-2))),
                    createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-3))),
                  ),
                ),
              ),
              createApplication(
                createBuiltin(Stdlib.Identity),
                createUnitList(
                  createTriple(
                    createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-4))),
                    createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-5))),
                    createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-6))),
                  ),
                ),
              ),
              createApplication(
                createBuiltin(Stdlib.Identity),
                createUnitList(
                  createTriple(
                    createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-7))),
                    createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-8))),
                    createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-9))),
                  ),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[[1, 2, 3], [4, 5, 6], [7, 8, 9]]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Record)', (assert, {
      createApplication,
      createEmptyList,
      createBuiltin,
      createInt,
      createRecord,
      createString,
      createTriple,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(createRecord(createEmptyList(), createEmptyList())),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{}');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(
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
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(
            createRecord(
              createTriple(
                createApplication(
                  createBuiltin(Stdlib.Identity),
                  createUnitList(createString('foo')),
                ),
                createApplication(
                  createBuiltin(Stdlib.Identity),
                  createUnitList(createString('bar')),
                ),
                createApplication(
                  createBuiltin(Stdlib.Identity),
                  createUnitList(createString('baz')),
                ),
              ),
              createTriple(
                createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-3))),
                createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-4))),
                createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-5))),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{ "foo": 3, "bar": 4, "baz": 5 }');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(
            createRecord(
              createTriple(
                createApplication(
                  createBuiltin(Stdlib.Identity),
                  createUnitList(createString('foo')),
                ),
                createApplication(
                  createBuiltin(Stdlib.Identity),
                  createUnitList(createString('bar')),
                ),
                createApplication(
                  createBuiltin(Stdlib.Identity),
                  createUnitList(createString('baz')),
                ),
              ),
              createTriple(
                createApplication(
                  createBuiltin(Stdlib.Identity),
                  createUnitList(
                    createTriple(
                      createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-1))),
                      createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-2))),
                      createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-3))),
                    ),
                  ),
                ),
                createApplication(
                  createBuiltin(Stdlib.Identity),
                  createUnitList(
                    createTriple(
                      createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-4))),
                      createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-5))),
                      createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-6))),
                    ),
                  ),
                ),
                createApplication(
                  createBuiltin(Stdlib.Identity),
                  createUnitList(
                    createTriple(
                      createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-7))),
                      createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-8))),
                      createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-9))),
                    ),
                  ),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{ "foo": [1, 2, 3], "bar": [4, 5, 6], "baz": [7, 8, 9] }',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Hashmap)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createHashmap,
      createString,
      createTriple,
      createUnitList,
      evaluate,
      format,
      getHashmapNumEntries,
      getHashmapValue,
      isHashmap,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(createHashmap([])),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isHashmap(result));
        assert.strictEqual(getHashmapNumEntries(result), 0);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(
            createHashmap([
              [createString('foo'), createInt(3)],
              [createString('bar'), createInt(4)],
              [createString('baz'), createInt(5)],
            ]),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isHashmap(result));
        assert.strictEqual(getHashmapNumEntries(result), 3);
        assert.strictEqual(format(getHashmapValue(result, createString('foo'))), '3');
        assert.strictEqual(format(getHashmapValue(result, createString('bar'))), '4');
        assert.strictEqual(format(getHashmapValue(result, createString('baz'))), '5');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(
            createHashmap([
              [
                createApplication(
                  createBuiltin(Stdlib.Identity),
                  createUnitList(createString('foo')),
                ),
                createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-3))),
              ],
              [
                createApplication(
                  createBuiltin(Stdlib.Identity),
                  createUnitList(createString('bar')),
                ),
                createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-4))),
              ],
              [
                createApplication(
                  createBuiltin(Stdlib.Identity),
                  createUnitList(createString('baz')),
                ),
                createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-5))),
              ],
            ]),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isHashmap(result));
        assert.strictEqual(getHashmapNumEntries(result), 3);
        assert.strictEqual(format(getHashmapValue(result, createString('foo'))), '3');
        assert.strictEqual(format(getHashmapValue(result, createString('bar'))), '4');
        assert.strictEqual(format(getHashmapValue(result, createString('baz'))), '5');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(
            createHashmap([
              [
                createApplication(
                  createBuiltin(Stdlib.Identity),
                  createUnitList(createString('foo')),
                ),
                createApplication(
                  createBuiltin(Stdlib.Identity),
                  createUnitList(
                    createTriple(
                      createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-1))),
                      createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-2))),
                      createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-3))),
                    ),
                  ),
                ),
              ],
              [
                createApplication(
                  createBuiltin(Stdlib.Identity),
                  createUnitList(createString('bar')),
                ),
                createApplication(
                  createBuiltin(Stdlib.Identity),
                  createUnitList(
                    createTriple(
                      createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-4))),
                      createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-5))),
                      createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-6))),
                    ),
                  ),
                ),
              ],
              [
                createApplication(
                  createBuiltin(Stdlib.Identity),
                  createUnitList(createString('baz')),
                ),
                createApplication(
                  createBuiltin(Stdlib.Identity),
                  createUnitList(
                    createTriple(
                      createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-7))),
                      createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-8))),
                      createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-9))),
                    ),
                  ),
                ),
              ],
            ]),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isHashmap(result));
        assert.strictEqual(getHashmapNumEntries(result), 3);
        assert.strictEqual(format(getHashmapValue(result, createString('foo'))), '[1, 2, 3]');
        assert.strictEqual(format(getHashmapValue(result, createString('bar'))), '[4, 5, 6]');
        assert.strictEqual(format(getHashmapValue(result, createString('baz'))), '[7, 8, 9]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Hashset)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createHashset,
      createString,
      createTriple,
      createUnitList,
      evaluate,
      format,
      getHashsetNumEntries,
      hasHashsetValue,
      isHashset,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(createHashset([])),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isHashset(result));
        assert.strictEqual(getHashsetNumEntries(result), 0);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(
            createHashset([createString('foo'), createString('bar'), createString('baz')]),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isHashset(result));
        assert.strictEqual(getHashsetNumEntries(result), 3);
        assert.strictEqual(hasHashsetValue(result, createString('foo')), true);
        assert.strictEqual(hasHashsetValue(result, createString('bar')), true);
        assert.strictEqual(hasHashsetValue(result, createString('baz')), true);
        assert.strictEqual(hasHashsetValue(result, createString('qux')), false);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(
            createHashset([
              createApplication(
                createBuiltin(Stdlib.Identity),
                createUnitList(createString('foo')),
              ),
              createApplication(
                createBuiltin(Stdlib.Identity),
                createUnitList(createString('bar')),
              ),
              createApplication(
                createBuiltin(Stdlib.Identity),
                createUnitList(createString('baz')),
              ),
            ]),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isHashset(result));
        assert.strictEqual(getHashsetNumEntries(result), 3);
        assert.strictEqual(hasHashsetValue(result, createString('foo')), true);
        assert.strictEqual(hasHashsetValue(result, createString('bar')), true);
        assert.strictEqual(hasHashsetValue(result, createString('baz')), true);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(
            createHashset([
              createApplication(
                createBuiltin(Stdlib.Identity),
                createUnitList(
                  createTriple(
                    createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-1))),
                    createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-2))),
                    createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-3))),
                  ),
                ),
              ),
              createApplication(
                createBuiltin(Stdlib.Identity),
                createUnitList(
                  createTriple(
                    createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-4))),
                    createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-5))),
                    createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-6))),
                  ),
                ),
              ),
              createApplication(
                createBuiltin(Stdlib.Identity),
                createUnitList(
                  createTriple(
                    createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-7))),
                    createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-8))),
                    createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-9))),
                  ),
                ),
              ),
            ]),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isHashset(result));
        assert.strictEqual(getHashsetNumEntries(result), 3);
        assert.strictEqual(
          hasHashsetValue(result, createTriple(createInt(1), createInt(2), createInt(3))),
          true,
        );
        assert.strictEqual(
          hasHashsetValue(result, createTriple(createInt(4), createInt(5), createInt(6))),
          true,
        );
        assert.strictEqual(
          hasHashsetValue(result, createTriple(createInt(7), createInt(8), createInt(9))),
          true,
        );
        assert.strictEqual(
          hasHashsetValue(
            result,
            createTriple(
              createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-1))),
              createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-2))),
              createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-3))),
            ),
          ),
          false,
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Tree)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createTree,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(createTree(NULL, NULL)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '(NULL . NULL)');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(createTree(createInt(3), createInt(4))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '(3 . 4)');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(
            createTree(
              createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-3))),
              createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-4))),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '(3 . 4)');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(
            createTree(
              createTree(
                createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-1))),
                createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-2))),
              ),
              createTree(
                createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-3))),
                createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-4))),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '((1 . 2) . (3 . 4))');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Iterator)', (assert, {
      createApplication,
      createEmptyIterator,
      createRangeIterator,
      createBuiltin,
      createLambda,
      createMapIterator,
      createPair,
      createUnitList,
      createVariable,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(createEmptyIterator()),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(createRangeIterator(3, 3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(createMapIterator(createRangeIterator(-3, 3), createBuiltin(Stdlib.Abs))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 2, 1]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveDeep),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Identity),
              createUnitList(
                createMapIterator(
                  createRangeIterator(3, 3),
                  createLambda(
                    1,
                    createApplication(
                      createBuiltin(Stdlib.Identity),
                      createUnitList(
                        createMapIterator(
                          createRangeIterator(0, 3),
                          createLambda(
                            1,
                            createApplication(
                              createBuiltin(Stdlib.Add),
                              createPair(createVariable(1), createVariable(0)),
                            ),
                          ),
                        ),
                      ),
                    ),
                  ),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[[3, 4, 5], [4, 5, 6], [5, 6, 7]]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};

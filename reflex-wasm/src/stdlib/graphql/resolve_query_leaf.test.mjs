// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_ResolveQueryLeaf', (test) => {
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
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(createNil()),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'null');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(
            createApplication(createBuiltin(Stdlib.Identity), createUnitList(createNil())),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'null');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Boolean)', (assert, {
      createApplication,
      createBoolean,
      createBuiltin,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(createBoolean(true)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'true');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(
            createApplication(createBuiltin(Stdlib.Identity), createUnitList(createBoolean(true))),
          ),
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
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(createInt(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(
            createApplication(createBuiltin(Stdlib.Identity), createUnitList(createInt(3))),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Float)', (assert, {
      createApplication,
      createBuiltin,
      createFloat,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(createFloat(3.142)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3.142');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(
            createApplication(createBuiltin(Stdlib.Identity), createUnitList(createFloat(3.142))),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3.142');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(String)', (assert, {
      createApplication,
      createBuiltin,
      createString,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(createString('foo')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"foo"');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(
            createApplication(createBuiltin(Stdlib.Identity), createUnitList(createString('foo'))),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"foo"');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(List)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createEmptyList,
      createUnitList,
      createTriple,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(createEmptyList()),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(
            createApplication(createBuiltin(Stdlib.Identity), createUnitList(createEmptyList())),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(createTriple(createInt(3), createInt(4), createInt(5))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Identity),
              createUnitList(
                createTriple(
                  createApplication(createBuiltin(Stdlib.Identity), createUnitList(createInt(3))),
                  createApplication(createBuiltin(Stdlib.Identity), createUnitList(createInt(4))),
                  createApplication(createBuiltin(Stdlib.Identity), createUnitList(createInt(5))),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(
            createTriple(
              createTriple(createInt(1), createInt(2), createInt(3)),
              createTriple(createInt(4), createInt(5), createInt(6)),
              createTriple(createInt(7), createInt(8), createInt(9)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[[1, 2, 3], [4, 5, 6], [7, 8, 9]]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Identity),
              createUnitList(
                createTriple(
                  createApplication(
                    createBuiltin(Stdlib.Identity),
                    createUnitList(
                      createTriple(
                        createApplication(
                          createBuiltin(Stdlib.Identity),
                          createUnitList(createInt(1)),
                        ),
                        createApplication(
                          createBuiltin(Stdlib.Identity),
                          createUnitList(createInt(2)),
                        ),
                        createApplication(
                          createBuiltin(Stdlib.Identity),
                          createUnitList(createInt(3)),
                        ),
                      ),
                    ),
                  ),
                  createApplication(
                    createBuiltin(Stdlib.Identity),
                    createUnitList(
                      createTriple(
                        createApplication(
                          createBuiltin(Stdlib.Identity),
                          createUnitList(createInt(4)),
                        ),
                        createApplication(
                          createBuiltin(Stdlib.Identity),
                          createUnitList(createInt(5)),
                        ),
                        createApplication(
                          createBuiltin(Stdlib.Identity),
                          createUnitList(createInt(6)),
                        ),
                      ),
                    ),
                  ),
                  createApplication(
                    createBuiltin(Stdlib.Identity),
                    createUnitList(
                      createTriple(
                        createApplication(
                          createBuiltin(Stdlib.Identity),
                          createUnitList(createInt(7)),
                        ),
                        createApplication(
                          createBuiltin(Stdlib.Identity),
                          createUnitList(createInt(8)),
                        ),
                        createApplication(
                          createBuiltin(Stdlib.Identity),
                          createUnitList(createInt(9)),
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
        assert.strictEqual(format(result), '[[1, 2, 3], [4, 5, 6], [7, 8, 9]]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Iterator)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyIterator,
      createMapIterator,
      createRangeIterator,
      createTriple,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(createEmptyIterator()),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Identity),
              createUnitList(createEmptyIterator()),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(createRangeIterator(3, 3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Identity),
              createUnitList(
                createMapIterator(createRangeIterator(3, 3), createBuiltin(Stdlib.Identity)),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(
            createTriple(
              createRangeIterator(1, 3),
              createRangeIterator(4, 3),
              createRangeIterator(7, 3),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[[1, 2, 3], [4, 5, 6], [7, 8, 9]]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Identity),
              createUnitList(
                createTriple(
                  createApplication(
                    createBuiltin(Stdlib.Identity),
                    createUnitList(
                      createMapIterator(createRangeIterator(1, 3), createBuiltin(Stdlib.Identity)),
                    ),
                  ),
                  createApplication(
                    createBuiltin(Stdlib.Identity),
                    createUnitList(
                      createMapIterator(createRangeIterator(4, 3), createBuiltin(Stdlib.Identity)),
                    ),
                  ),
                  createApplication(
                    createBuiltin(Stdlib.Identity),
                    createUnitList(
                      createMapIterator(createRangeIterator(7, 3), createBuiltin(Stdlib.Identity)),
                    ),
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

    test('(Lambda)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createLambda,
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
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(createLambda(0, createString('foo'))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"foo"');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(createLambda(0, createInt(3))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(createLambda(0, createTriple(createInt(3), createInt(4), createInt(5)))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(
            createLambda(
              0,
              createTriple(
                createLambda(0, createInt(3)),
                createLambda(0, createInt(4)),
                createLambda(0, createInt(5)),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Record)', (assert, {
      createApplication,
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
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(
            createRecord(
              createTriple(createString('foo'), createString('bar'), createString('baz')),
              createTriple(createInt(3), createInt(4), createInt(5)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<InvalidFunctionArgsCondition:ResolveQueryLeaf({ "foo": 3, "bar": 4, "baz": 5 })>}',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Hashmap)', (assert, {
      createApplication,
      createBuiltin,
      createHashmap,
      createInt,
      createString,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(
            createHashmap([
              [createString('foo'), createInt(3)],
              [createString('bar'), createInt(4)],
              [createString('baz'), createInt(5)],
            ]),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<InvalidFunctionArgsCondition:ResolveQueryLeaf(Map(3))>}',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Hashset)', (assert, {
      createApplication,
      createBuiltin,
      createHashset,
      createInt,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(createHashset([createInt(3), createInt(4), createInt(5)])),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<InvalidFunctionArgsCondition:ResolveQueryLeaf(Set(3))>}',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Tree)', (assert, {
      createApplication,
      createBuiltin,
      createTree,
      createInt,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryLeaf),
          createUnitList(createTree(createInt(3), createInt(4))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<InvalidFunctionArgsCondition:ResolveQueryLeaf((3 . 4))>}',
        );
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};

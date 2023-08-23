// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_ResolveQueryBranch', (test) => {
    test('(Nil)', (assert, {
      createApplication,
      createBuiltin,
      createNil,
      createPair,
      createUnitList,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryBranch),
          createPair(createNil(), createBuiltin(Stdlib.Identity)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'null');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryBranch),
          createPair(
            createApplication(createBuiltin(Stdlib.Identity), createUnitList(createNil())),
            createApplication(
              createBuiltin(Stdlib.Identity),
              createUnitList(createBuiltin(Stdlib.Identity)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'null');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Record)', (assert, {
      createApplication,
      createBuiltin,
      createConstructor,
      createInt,
      createLambda,
      createPair,
      createRecord,
      createString,
      createTriple,
      createUnitList,
      createVariable,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      const shape = createLambda(
        1,
        createApplication(
          createBuiltin(Stdlib.Apply),
          createPair(
            createConstructor(createPair(createString('first'), createString('second'))),
            createApplication(
              createBuiltin(Stdlib.ResolveList),
              createUnitList(
                createPair(
                  createApplication(
                    createBuiltin(Stdlib.Get),
                    createPair(createVariable(0), createString('foo')),
                  ),
                  createApplication(
                    createBuiltin(Stdlib.Get),
                    createPair(createVariable(0), createString('baz')),
                  ),
                ),
              ),
            ),
          ),
        ),
      );
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryBranch),
          createPair(
            createRecord(
              createTriple(createString('foo'), createString('bar'), createString('baz')),
              createTriple(createInt(3), createInt(4), createInt(5)),
            ),
            shape,
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{ "first": 3, "second": 5 }');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryBranch),
          createPair(
            createApplication(
              createBuiltin(Stdlib.Identity),
              createUnitList(
                createRecord(
                  createTriple(createString('foo'), createString('bar'), createString('baz')),
                  createTriple(createInt(3), createInt(4), createInt(5)),
                ),
              ),
            ),
            createApplication(createBuiltin(Stdlib.Identity), createUnitList(shape)),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{ "first": 3, "second": 5 }');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(List)', (assert, {
      createApplication,
      createBuiltin,
      createConstructor,
      createInt,
      createEmptyList,
      createLambda,
      createRecord,
      createPair,
      createUnitList,
      createString,
      createTriple,
      createVariable,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      const shape = createLambda(
        1,
        createApplication(
          createBuiltin(Stdlib.Apply),
          createPair(
            createConstructor(createPair(createString('first'), createString('second'))),
            createApplication(
              createBuiltin(Stdlib.ResolveList),
              createUnitList(
                createPair(
                  createApplication(
                    createBuiltin(Stdlib.Get),
                    createPair(createVariable(0), createString('foo')),
                  ),
                  createApplication(
                    createBuiltin(Stdlib.Get),
                    createPair(createVariable(0), createString('baz')),
                  ),
                ),
              ),
            ),
          ),
        ),
      );
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryBranch),
          createPair(createEmptyList(), shape),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryBranch),
          createPair(
            createApplication(createBuiltin(Stdlib.Identity), createUnitList(createEmptyList())),
            createApplication(createBuiltin(Stdlib.Identity), createUnitList(shape)),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryBranch),
          createPair(
            createTriple(
              createRecord(
                createTriple(createString('foo'), createString('bar'), createString('baz')),
                createTriple(createInt(1), createInt(2), createInt(3)),
              ),
              createRecord(
                createTriple(createString('foo'), createString('bar'), createString('baz')),
                createTriple(createInt(4), createInt(5), createInt(6)),
              ),
              createRecord(
                createTriple(createString('foo'), createString('bar'), createString('baz')),
                createTriple(createInt(7), createInt(8), createInt(9)),
              ),
            ),
            shape,
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '[{ "first": 1, "second": 3 }, { "first": 4, "second": 6 }, { "first": 7, "second": 9 }]',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryBranch),
          createPair(
            createApplication(
              createBuiltin(Stdlib.Identity),
              createUnitList(
                createTriple(
                  createApplication(
                    createBuiltin(Stdlib.Identity),
                    createUnitList(
                      createRecord(
                        createTriple(createString('foo'), createString('bar'), createString('baz')),
                        createTriple(createInt(1), createInt(2), createInt(3)),
                      ),
                    ),
                  ),
                  createApplication(
                    createBuiltin(Stdlib.Identity),
                    createUnitList(
                      createRecord(
                        createTriple(createString('foo'), createString('bar'), createString('baz')),
                        createTriple(createInt(4), createInt(5), createInt(6)),
                      ),
                    ),
                  ),
                  createApplication(
                    createBuiltin(Stdlib.Identity),
                    createUnitList(
                      createRecord(
                        createTriple(createString('foo'), createString('bar'), createString('baz')),
                        createTriple(createInt(7), createInt(8), createInt(9)),
                      ),
                    ),
                  ),
                ),
              ),
            ),
            createApplication(createBuiltin(Stdlib.Identity), createUnitList(shape)),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '[{ "first": 1, "second": 3 }, { "first": 4, "second": 6 }, { "first": 7, "second": 9 }]',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryBranch),
          createPair(
            createTriple(
              createTriple(
                createRecord(
                  createTriple(createString('foo'), createString('bar'), createString('baz')),
                  createTriple(createInt(1), createInt(2), createInt(3)),
                ),
                createRecord(
                  createTriple(createString('foo'), createString('bar'), createString('baz')),
                  createTriple(createInt(4), createInt(5), createInt(6)),
                ),
                createRecord(
                  createTriple(createString('foo'), createString('bar'), createString('baz')),
                  createTriple(createInt(7), createInt(8), createInt(9)),
                ),
              ),
              createTriple(
                createRecord(
                  createTriple(createString('foo'), createString('bar'), createString('baz')),
                  createTriple(createInt(10), createInt(11), createInt(12)),
                ),
                createRecord(
                  createTriple(createString('foo'), createString('bar'), createString('baz')),
                  createTriple(createInt(13), createInt(14), createInt(15)),
                ),
                createRecord(
                  createTriple(createString('foo'), createString('bar'), createString('baz')),
                  createTriple(createInt(16), createInt(17), createInt(18)),
                ),
              ),
              createTriple(
                createRecord(
                  createTriple(createString('foo'), createString('bar'), createString('baz')),
                  createTriple(createInt(19), createInt(20), createInt(21)),
                ),
                createRecord(
                  createTriple(createString('foo'), createString('bar'), createString('baz')),
                  createTriple(createInt(22), createInt(23), createInt(24)),
                ),
                createRecord(
                  createTriple(createString('foo'), createString('bar'), createString('baz')),
                  createTriple(createInt(25), createInt(26), createInt(27)),
                ),
              ),
            ),
            shape,
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '[[{ "first": 1, "second": 3 }, { "first": 4, "second": 6 }, { "first": 7, "second": 9 }], [{ "first": 10, "second": 12 }, { "first": 13, "second": 15 }, { "first": 16, "second": 18 }], [{ "first": 19, "second": 21 }, { "first": 22, "second": 24 }, { "first": 25, "second": 27 }]]',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryBranch),
          createPair(
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
                          createUnitList(
                            createRecord(
                              createTriple(
                                createString('foo'),
                                createString('bar'),
                                createString('baz'),
                              ),
                              createTriple(createInt(1), createInt(2), createInt(3)),
                            ),
                          ),
                        ),
                        createApplication(
                          createBuiltin(Stdlib.Identity),
                          createUnitList(
                            createRecord(
                              createTriple(
                                createString('foo'),
                                createString('bar'),
                                createString('baz'),
                              ),
                              createTriple(createInt(4), createInt(5), createInt(6)),
                            ),
                          ),
                        ),
                        createApplication(
                          createBuiltin(Stdlib.Identity),
                          createUnitList(
                            createRecord(
                              createTriple(
                                createString('foo'),
                                createString('bar'),
                                createString('baz'),
                              ),
                              createTriple(createInt(7), createInt(8), createInt(9)),
                            ),
                          ),
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
                          createUnitList(
                            createRecord(
                              createTriple(
                                createString('foo'),
                                createString('bar'),
                                createString('baz'),
                              ),
                              createTriple(createInt(10), createInt(11), createInt(12)),
                            ),
                          ),
                        ),
                        createApplication(
                          createBuiltin(Stdlib.Identity),
                          createUnitList(
                            createRecord(
                              createTriple(
                                createString('foo'),
                                createString('bar'),
                                createString('baz'),
                              ),
                              createTriple(createInt(13), createInt(14), createInt(15)),
                            ),
                          ),
                        ),
                        createApplication(
                          createBuiltin(Stdlib.Identity),
                          createUnitList(
                            createRecord(
                              createTriple(
                                createString('foo'),
                                createString('bar'),
                                createString('baz'),
                              ),
                              createTriple(createInt(16), createInt(17), createInt(18)),
                            ),
                          ),
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
                          createUnitList(
                            createRecord(
                              createTriple(
                                createString('foo'),
                                createString('bar'),
                                createString('baz'),
                              ),
                              createTriple(createInt(19), createInt(20), createInt(21)),
                            ),
                          ),
                        ),
                        createApplication(
                          createBuiltin(Stdlib.Identity),
                          createUnitList(
                            createRecord(
                              createTriple(
                                createString('foo'),
                                createString('bar'),
                                createString('baz'),
                              ),
                              createTriple(createInt(22), createInt(23), createInt(24)),
                            ),
                          ),
                        ),
                        createApplication(
                          createBuiltin(Stdlib.Identity),
                          createUnitList(
                            createRecord(
                              createTriple(
                                createString('foo'),
                                createString('bar'),
                                createString('baz'),
                              ),
                              createTriple(createInt(25), createInt(26), createInt(27)),
                            ),
                          ),
                        ),
                      ),
                    ),
                  ),
                ),
              ),
            ),
            shape,
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '[[{ "first": 1, "second": 3 }, { "first": 4, "second": 6 }, { "first": 7, "second": 9 }], [{ "first": 10, "second": 12 }, { "first": 13, "second": 15 }, { "first": 16, "second": 18 }], [{ "first": 19, "second": 21 }, { "first": 22, "second": 24 }, { "first": 25, "second": 27 }]]',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Iterator)', (assert, {
      createApplication,
      createBuiltin,
      createConstructor,
      createInt,
      createEmptyIterator,
      createLambda,
      createMapIterator,
      createPair,
      createUnitList,
      createString,
      createTriple,
      createRangeIterator,
      createVariable,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      const shape = createLambda(
        1,
        createApplication(
          createBuiltin(Stdlib.Apply),
          createPair(
            createConstructor(createPair(createString('first'), createString('second'))),
            createApplication(
              createBuiltin(Stdlib.ResolveList),
              createUnitList(
                createPair(
                  createApplication(
                    createBuiltin(Stdlib.Get),
                    createPair(createVariable(0), createString('foo')),
                  ),
                  createApplication(
                    createBuiltin(Stdlib.Get),
                    createPair(createVariable(0), createString('baz')),
                  ),
                ),
              ),
            ),
          ),
        ),
      );
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryBranch),
          createPair(createEmptyIterator(), shape),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryBranch),
          createPair(
            createApplication(
              createBuiltin(Stdlib.Identity),
              createUnitList(createEmptyIterator()),
            ),
            createApplication(createBuiltin(Stdlib.Identity), createUnitList(shape)),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryBranch),
          createPair(
            createMapIterator(
              createRangeIterator(0, 3),
              createLambda(
                1,
                createApplication(
                  createBuiltin(Stdlib.Apply),
                  createPair(
                    createConstructor(
                      createTriple(createString('foo'), createString('bar'), createString('baz')),
                    ),
                    createApplication(
                      createBuiltin(Stdlib.ResolveList),
                      createUnitList(
                        createTriple(
                          createApplication(
                            createBuiltin(Stdlib.Add),
                            createPair(
                              createApplication(
                                createBuiltin(Stdlib.Multiply),
                                createPair(createVariable(0), createInt(3)),
                              ),
                              createInt(1),
                            ),
                          ),
                          createApplication(
                            createBuiltin(Stdlib.Add),
                            createPair(
                              createApplication(
                                createBuiltin(Stdlib.Multiply),
                                createPair(createVariable(0), createInt(3)),
                              ),
                              createInt(2),
                            ),
                          ),
                          createApplication(
                            createBuiltin(Stdlib.Add),
                            createPair(
                              createApplication(
                                createBuiltin(Stdlib.Multiply),
                                createPair(createVariable(0), createInt(3)),
                              ),
                              createInt(3),
                            ),
                          ),
                        ),
                      ),
                    ),
                  ),
                ),
              ),
            ),
            shape,
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '[{ "first": 1, "second": 3 }, { "first": 4, "second": 6 }, { "first": 7, "second": 9 }]',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryBranch),
          createPair(
            createMapIterator(
              createRangeIterator(0, 3),
              createLambda(
                1,
                createMapIterator(
                  createRangeIterator(0, 3),
                  createLambda(
                    1,
                    createApplication(
                      createBuiltin(Stdlib.Apply),
                      createPair(
                        createConstructor(
                          createTriple(
                            createString('foo'),
                            createString('bar'),
                            createString('baz'),
                          ),
                        ),
                        createApplication(
                          createBuiltin(Stdlib.ResolveList),
                          createUnitList(
                            createTriple(
                              createApplication(
                                createBuiltin(Stdlib.Add),
                                createPair(
                                  createApplication(
                                    createBuiltin(Stdlib.Add),
                                    createPair(
                                      createApplication(
                                        createBuiltin(Stdlib.Multiply),
                                        createPair(createVariable(0), createInt(3)),
                                      ),
                                      createApplication(
                                        createBuiltin(Stdlib.Multiply),
                                        createPair(createVariable(1), createInt(9)),
                                      ),
                                    ),
                                  ),
                                  createInt(1),
                                ),
                              ),
                              createApplication(
                                createBuiltin(Stdlib.Add),
                                createPair(
                                  createApplication(
                                    createBuiltin(Stdlib.Add),
                                    createPair(
                                      createApplication(
                                        createBuiltin(Stdlib.Multiply),
                                        createPair(createVariable(0), createInt(3)),
                                      ),
                                      createApplication(
                                        createBuiltin(Stdlib.Multiply),
                                        createPair(createVariable(1), createInt(9)),
                                      ),
                                    ),
                                  ),
                                  createInt(2),
                                ),
                              ),
                              createApplication(
                                createBuiltin(Stdlib.Add),
                                createPair(
                                  createApplication(
                                    createBuiltin(Stdlib.Add),
                                    createPair(
                                      createApplication(
                                        createBuiltin(Stdlib.Multiply),
                                        createPair(createVariable(0), createInt(3)),
                                      ),
                                      createApplication(
                                        createBuiltin(Stdlib.Multiply),
                                        createPair(createVariable(1), createInt(9)),
                                      ),
                                    ),
                                  ),
                                  createInt(3),
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
            ),
            shape,
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '[[{ "first": 1, "second": 3 }, { "first": 4, "second": 6 }, { "first": 7, "second": 9 }], [{ "first": 10, "second": 12 }, { "first": 13, "second": 15 }, { "first": 16, "second": 18 }], [{ "first": 19, "second": 21 }, { "first": 22, "second": 24 }, { "first": 25, "second": 27 }]]',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Lambda)', (assert, {
      createApplication,
      createBuiltin,
      createConstructor,
      createInt,
      createLambda,
      createPair,
      createRecord,
      createString,
      createTriple,
      createUnitList,
      createVariable,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      const shape = createLambda(
        1,
        createApplication(
          createBuiltin(Stdlib.Apply),
          createPair(
            createConstructor(createPair(createString('first'), createString('second'))),
            createApplication(
              createBuiltin(Stdlib.ResolveList),
              createUnitList(
                createPair(
                  createApplication(
                    createBuiltin(Stdlib.Get),
                    createPair(createVariable(0), createString('foo')),
                  ),
                  createApplication(
                    createBuiltin(Stdlib.Get),
                    createPair(createVariable(0), createString('baz')),
                  ),
                ),
              ),
            ),
          ),
        ),
      );
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryBranch),
          createPair(
            createLambda(
              0,
              createRecord(
                createTriple(createString('foo'), createString('bar'), createString('baz')),
                createTriple(createInt(3), createInt(4), createInt(5)),
              ),
            ),
            shape,
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{ "first": 3, "second": 5 }');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveQueryBranch),
          createPair(
            createApplication(
              createBuiltin(Stdlib.Identity),
              createUnitList(
                createLambda(
                  0,
                  createRecord(
                    createTriple(createString('foo'), createString('bar'), createString('baz')),
                    createTriple(createInt(3), createInt(4), createInt(5)),
                  ),
                ),
              ),
            ),
            createApplication(createBuiltin(Stdlib.Identity), createUnitList(shape)),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{ "first": 3, "second": 5 }');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Hashmap)', (assert, {
      createApplication,
      createBuiltin,
      createConstructor,
      createHashmap,
      createLambda,
      createInt,
      createPair,
      createString,
      createUnitList,
      createVariable,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      const shape = createLambda(
        1,
        createApplication(
          createBuiltin(Stdlib.Apply),
          createPair(
            createConstructor(createPair(createString('first'), createString('second'))),
            createApplication(
              createBuiltin(Stdlib.ResolveList),
              createUnitList(
                createPair(
                  createApplication(
                    createBuiltin(Stdlib.Get),
                    createPair(createVariable(0), createString('foo')),
                  ),
                  createApplication(
                    createBuiltin(Stdlib.Get),
                    createPair(createVariable(0), createString('baz')),
                  ),
                ),
              ),
            ),
          ),
        ),
      );
      const expression = createApplication(
        createBuiltin(Stdlib.ResolveQueryBranch),
        createPair(
          createHashmap([
            [createString('foo'), createInt(3)],
            [createString('bar'), createInt(4)],
            [createString('baz'), createInt(5)],
          ]),
          shape,
        ),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(
        format(result),
        `{<InvalidFunctionArgsCondition:ResolveQueryBranch(Map(3), ${format(shape)})>}`,
      );
      assert.deepEqual(getStateDependencies(dependencies), []);
    });

    test('(Hashset)', (assert, {
      createApplication,
      createBuiltin,
      createConstructor,
      createHashset,
      createInt,
      createLambda,
      createPair,
      createString,
      createUnitList,
      createVariable,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      const shape = createLambda(
        1,
        createApplication(
          createBuiltin(Stdlib.Apply),
          createPair(
            createConstructor(createPair(createString('first'), createString('second'))),
            createApplication(
              createBuiltin(Stdlib.ResolveList),
              createUnitList(
                createPair(
                  createApplication(
                    createBuiltin(Stdlib.Get),
                    createPair(createVariable(0), createString('foo')),
                  ),
                  createApplication(
                    createBuiltin(Stdlib.Get),
                    createPair(createVariable(0), createString('baz')),
                  ),
                ),
              ),
            ),
          ),
        ),
      );
      const expression = createApplication(
        createBuiltin(Stdlib.ResolveQueryBranch),
        createPair(createHashset([createInt(3), createInt(4), createInt(5)]), shape),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(
        format(result),
        `{<InvalidFunctionArgsCondition:ResolveQueryBranch(Set(3), ${format(shape)})>}`,
      );
      assert.deepEqual(getStateDependencies(dependencies), []);
    });

    test('(Tree)', (assert, {
      createApplication,
      createBuiltin,
      createConstructor,
      createLambda,
      createPair,
      createString,
      createTree,
      createInt,
      createUnitList,
      createVariable,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      const shape = createLambda(
        1,
        createApplication(
          createBuiltin(Stdlib.Apply),
          createPair(
            createConstructor(createPair(createString('first'), createString('second'))),
            createApplication(
              createBuiltin(Stdlib.ResolveList),
              createUnitList(
                createPair(
                  createApplication(
                    createBuiltin(Stdlib.Get),
                    createPair(createVariable(0), createString('foo')),
                  ),
                  createApplication(
                    createBuiltin(Stdlib.Get),
                    createPair(createVariable(0), createString('baz')),
                  ),
                ),
              ),
            ),
          ),
        ),
      );
      const expression = createApplication(
        createBuiltin(Stdlib.ResolveQueryBranch),
        createPair(createTree(createInt(3), createInt(4)), shape),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.strictEqual(
        format(result),
        `{<InvalidFunctionArgsCondition:ResolveQueryBranch((3 . 4), ${format(shape)})>}`,
      );
      assert.deepEqual(getStateDependencies(dependencies), []);
    });
  });
};

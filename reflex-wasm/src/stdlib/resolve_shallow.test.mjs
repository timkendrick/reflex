// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_ResolveShallow', (test) => {
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
        createBuiltin(Stdlib.ResolveShallow),
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
          createBuiltin(Stdlib.ResolveShallow),
          createUnitList(createBoolean(false)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), 'false');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveShallow),
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
          createBuiltin(Stdlib.ResolveShallow),
          createUnitList(createInt(0)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '0');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveShallow),
          createUnitList(createInt(3)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveShallow),
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
          createBuiltin(Stdlib.ResolveShallow),
          createUnitList(createEmptyList()),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveShallow),
          createUnitList(createTriple(createInt(3), createInt(4), createInt(5))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveShallow),
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
          createBuiltin(Stdlib.ResolveShallow),
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
        assert.strictEqual(
          format(result),
          '[[Abs(-1), Abs(-2), Abs(-3)], [Abs(-4), Abs(-5), Abs(-6)], [Abs(-7), Abs(-8), Abs(-9)]]',
        );
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
          createBuiltin(Stdlib.ResolveShallow),
          createUnitList(createRecord(createEmptyList(), createEmptyList())),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{}');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveShallow),
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
          createBuiltin(Stdlib.ResolveShallow),
          createUnitList(
            createRecord(
              createTriple(
                createApplication(createBuiltin(Stdlib.Identity), createUnitList(createString('foo'))),
                createApplication(createBuiltin(Stdlib.Identity), createUnitList(createString('bar'))),
                createApplication(createBuiltin(Stdlib.Identity), createUnitList(createString('baz'))),
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
          createBuiltin(Stdlib.ResolveShallow),
          createUnitList(
            createRecord(
              createTriple(
                createApplication(createBuiltin(Stdlib.Identity), createUnitList(createString('foo'))),
                createApplication(createBuiltin(Stdlib.Identity), createUnitList(createString('bar'))),
                createApplication(createBuiltin(Stdlib.Identity), createUnitList(createString('baz'))),
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
          '{ "foo": [Abs(-1), Abs(-2), Abs(-3)], "bar": [Abs(-4), Abs(-5), Abs(-6)], "baz": [Abs(-7), Abs(-8), Abs(-9)] }',
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
          createBuiltin(Stdlib.ResolveShallow),
          createUnitList(createHashmap([])),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.ok(isHashmap(result));
        assert.strictEqual(getHashmapNumEntries(result), 0);
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveShallow),
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
          createBuiltin(Stdlib.ResolveShallow),
          createUnitList(
            createHashmap([
              [
                createApplication(createBuiltin(Stdlib.Identity), createUnitList(createString('foo'))),
                createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-3))),
              ],
              [
                createApplication(createBuiltin(Stdlib.Identity), createUnitList(createString('bar'))),
                createApplication(createBuiltin(Stdlib.Abs), createUnitList(createInt(-4))),
              ],
              [
                createApplication(createBuiltin(Stdlib.Identity), createUnitList(createString('baz'))),
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
          createBuiltin(Stdlib.ResolveShallow),
          createUnitList(
            createHashmap([
              [
                createApplication(createBuiltin(Stdlib.Identity), createUnitList(createString('foo'))),
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
                createApplication(createBuiltin(Stdlib.Identity), createUnitList(createString('bar'))),
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
                createApplication(createBuiltin(Stdlib.Identity), createUnitList(createString('baz'))),
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
        assert.strictEqual(format(getHashmapValue(result, createString('foo'))), '[Abs(-1), Abs(-2), Abs(-3)]');
        assert.strictEqual(format(getHashmapValue(result, createString('bar'))), '[Abs(-4), Abs(-5), Abs(-6)]');
        assert.strictEqual(format(getHashmapValue(result, createString('baz'))), '[Abs(-7), Abs(-8), Abs(-9)]');
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
          createBuiltin(Stdlib.ResolveShallow),
          createUnitList(createTree(NULL, NULL)),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '(NULL . NULL)');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveShallow),
          createUnitList(createTree(createInt(3), createInt(4))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '(3 . 4)');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveShallow),
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
          createBuiltin(Stdlib.ResolveShallow),
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
        assert.strictEqual(format(result), '((Abs(-1) . Abs(-2)) . (Abs(-3) . Abs(-4)))');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};

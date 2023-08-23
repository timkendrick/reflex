// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Term::Tree', (test) => {
    test('format', (assert, { createTree, createInt, format, NULL }) => {
      assert.strictEqual(format(createTree(NULL, NULL)), '(NULL . NULL)');
      assert.strictEqual(format(createTree(createInt(3), NULL)), '(3 . NULL)');
      assert.strictEqual(
        format(createTree(createInt(3), createTree(createInt(4), NULL))),
        '(3 . (4 . NULL))',
      );
      assert.strictEqual(format(createTree(createInt(3), createInt(4))), '(3 . 4)');
      assert.strictEqual(
        format(createTree(createTree(createInt(3), createInt(4)), createInt(5))),
        '((3 . 4) . 5)',
      );
      assert.strictEqual(
        format(createTree(createInt(3), createTree(createInt(4), createInt(5)))),
        '(3 . (4 . 5))',
      );
      assert.strictEqual(
        format(createTree(createInt(3), createTree(createInt(4), createTree(createInt(5), NULL)))),
        '(3 . (4 . (5 . NULL)))',
      );
      assert.strictEqual(
        format(
          createTree(
            createTree(createInt(1), createInt(2)),
            createTree(
              createTree(createInt(3), createInt(4)),
              createTree(createTree(createInt(5), createInt(6)), NULL),
            ),
          ),
        ),
        '((1 . 2) . ((3 . 4) . ((5 . 6) . NULL)))',
      );
    });

    test('hash', (assert, { createTree, createInt, hash, NULL }) => {
      assert.strictEqual(hash(createTree(NULL, NULL)), hash(createTree(NULL, NULL)));
      assert.strictEqual(
        hash(createTree(createInt(3), NULL)),
        hash(createTree(createInt(3), NULL)),
      );
      assert.strictEqual(
        hash(createTree(createInt(3), createInt(4))),
        hash(createTree(createInt(3), createInt(4))),
      );
      assert.strictEqual(
        hash(createTree(createTree(createInt(3), createInt(4)), createInt(5))),
        hash(createTree(createTree(createInt(3), createInt(4)), createInt(5))),
      );
      assert.strictEqual(
        hash(createTree(createInt(3), createTree(createInt(4), createInt(5)))),
        hash(createTree(createInt(3), createTree(createInt(4), createInt(5)))),
      );
      assert.notStrictEqual(
        hash(createTree(createInt(3), NULL)),
        hash(createTree(createInt(4), NULL)),
      );
      assert.notStrictEqual(
        hash(createTree(createInt(3), createInt(4))),
        hash(createTree(createInt(3), createInt(5))),
      );
      assert.notStrictEqual(
        hash(createTree(createInt(3), createTree(createInt(4), createInt(5)))),
        hash(createTree(createInt(3), createTree(createInt(4), createInt(6)))),
      );
      assert.notStrictEqual(
        hash(createTree(createTree(createInt(3), createInt(4)), createInt(5))),
        hash(createTree(createTree(createInt(3), createInt(4)), createInt(6))),
      );
      assert.notStrictEqual(hash(createTree(createInt(3), NULL)), hash(createInt(3)));
      assert.notStrictEqual(hash(createTree(createInt(3), createInt(4))), hash(createInt(3)));
      assert.notStrictEqual(hash(createTree(createInt(3), createInt(4))), hash(createInt(4)));
    });

    test('equals', (assert, { createTree, createInt, equals, NULL }) => {
      assert.ok(equals(createTree(NULL, NULL), createTree(NULL, NULL)));
      assert.ok(equals(createTree(createInt(3), NULL), createTree(createInt(3), NULL)));
      assert.ok(
        equals(createTree(createInt(3), createInt(4)), createTree(createInt(3), createInt(4))),
      );
      assert.ok(
        equals(
          createTree(createTree(createInt(3), createInt(4)), createInt(5)),
          createTree(createTree(createInt(3), createInt(4)), createInt(5)),
        ),
      );
      assert.ok(
        equals(
          createTree(createInt(3), createTree(createInt(4), createInt(5))),
          createTree(createInt(3), createTree(createInt(4), createInt(5))),
        ),
      );
      assert.ok(!equals(createTree(createInt(3), NULL), createTree(createInt(4), NULL)));
      assert.ok(
        !equals(createTree(createInt(3), createInt(4)), createTree(createInt(3), createInt(5))),
      );
      assert.ok(
        !equals(
          createTree(createInt(3), createTree(createInt(4), createInt(5))),
          createTree(createInt(3), createTree(createInt(4), createInt(6))),
        ),
      );
      assert.ok(
        !equals(
          createTree(createTree(createInt(3), createInt(4)), createInt(5)),
          createTree(createTree(createInt(3), createInt(4)), createInt(6)),
        ),
      );
      assert.ok(!equals(createTree(createInt(3), NULL), createInt(3)));
      assert.ok(!equals(createTree(createInt(3), createInt(4)), createInt(3)));
      assert.ok(!equals(createTree(createInt(3), createInt(4)), createInt(4)));
    });

    test('iteration', (assert, {
      createApplication,
      createTree,
      createBuiltin,
      createInt,
      createUnitList,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Iterate),
              createUnitList(createTree(NULL, NULL)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Iterate),
              createUnitList(createTree(createInt(3), NULL)),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Iterate),
              createUnitList(createTree(createInt(3), createTree(createInt(4), NULL))),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Iterate),
              createUnitList(
                createTree(createInt(3), createTree(createInt(4), createTree(createInt(5), NULL))),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Iterate),
              createUnitList(createTree(createInt(3), createInt(4))),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Iterate),
              createUnitList(createTree(createInt(3), createTree(createInt(4), createInt(5)))),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Iterate),
              createUnitList(
                createTree(createInt(3), createTree(createInt(4), createTree(createInt(5), NULL))),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Iterate),
              createUnitList(createTree(createTree(createInt(3), createInt(4)), createInt(5))),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Iterate),
              createUnitList(
                createTree(
                  createTree(createInt(3), createInt(4)),
                  createTree(createInt(5), createInt(6)),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5, 6]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Iterate),
              createUnitList(
                createTree(createTree(createInt(3), createTree(createInt(4), NULL)), createInt(5)),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.ResolveList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Iterate),
              createUnitList(
                createTree(
                  createTree(createInt(3), createTree(createInt(4), NULL)),
                  createTree(createInt(5), NULL),
                ),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};

// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Skip', (test) => {
    test('(Iterator, Int)', (assert, {
      createApplication,
      createEmptyList,
      createBuiltin,
      createInt,
      createPair,
      createTriple,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Skip),
              createPair(
                createApplication(
                  createBuiltin(Stdlib.Iterate),
                  createUnitList(createEmptyList()),
                ),
                createInt(0),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Skip),
              createPair(
                createApplication(
                  createBuiltin(Stdlib.Iterate),
                  createUnitList(createEmptyList()),
                ),
                createInt(3),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Skip),
              createPair(
                createApplication(
                  createBuiltin(Stdlib.Iterate),
                  createUnitList(createTriple(createInt(3), createInt(4), createInt(5))),
                ),
                createInt(0),
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
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Skip),
              createPair(
                createApplication(
                  createBuiltin(Stdlib.Iterate),
                  createUnitList(createTriple(createInt(3), createInt(4), createInt(5))),
                ),
                createInt(1),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[4, 5]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Skip),
              createPair(
                createApplication(
                  createBuiltin(Stdlib.Iterate),
                  createUnitList(createTriple(createInt(3), createInt(4), createInt(5))),
                ),
                createInt(2),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[5]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Skip),
              createPair(
                createApplication(
                  createBuiltin(Stdlib.Iterate),
                  createUnitList(createTriple(createInt(3), createInt(4), createInt(5))),
                ),
                createInt(3),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Skip),
              createPair(
                createApplication(
                  createBuiltin(Stdlib.Iterate),
                  createUnitList(createTriple(createInt(3), createInt(3), createInt(5))),
                ),
                createInt(4),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });

    test('(Iterator, Float)', (assert, {
      createApplication,
      createEmptyList,
      createBuiltin,
      createFloat,
      createInt,
      createPair,
      createTriple,
      createUnitList,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Skip),
              createPair(
                createApplication(
                  createBuiltin(Stdlib.Iterate),
                  createUnitList(createEmptyList()),
                ),
                createFloat(0.0),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Skip),
              createPair(
                createApplication(
                  createBuiltin(Stdlib.Iterate),
                  createUnitList(createEmptyList()),
                ),
                createFloat(0.0),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Skip),
              createPair(
                createApplication(
                  createBuiltin(Stdlib.Iterate),
                  createUnitList(createTriple(createInt(3), createInt(4), createInt(5))),
                ),
                createFloat(0.0),
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
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Skip),
              createPair(
                createApplication(
                  createBuiltin(Stdlib.Iterate),
                  createUnitList(createTriple(createInt(3), createInt(4), createInt(5))),
                ),
                createFloat(1.0),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[4, 5]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Skip),
              createPair(
                createApplication(
                  createBuiltin(Stdlib.Iterate),
                  createUnitList(createTriple(createInt(3), createInt(4), createInt(5))),
                ),
                createFloat(2.0),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[5]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Skip),
              createPair(
                createApplication(
                  createBuiltin(Stdlib.Iterate),
                  createUnitList(createTriple(createInt(3), createInt(4), createInt(5))),
                ),
                createFloat(3.0),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectList),
          createUnitList(
            createApplication(
              createBuiltin(Stdlib.Skip),
              createPair(
                createApplication(
                  createBuiltin(Stdlib.Iterate),
                  createUnitList(createTriple(createInt(3), createInt(3), createInt(5))),
                ),
                createFloat(4.0),
              ),
            ),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};

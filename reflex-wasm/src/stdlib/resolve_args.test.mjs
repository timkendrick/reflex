// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_ResolveArgs', (test) => {
    test('(Lambda)', (assert, {
      createApplication,
      createBuiltin,
      createEmptyList,
      createInt,
      createLambda,
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
          createApplication(
            createBuiltin(Stdlib.ResolveArgs),
            createUnitList(createLambda(0, createInt(3))),
          ),
          createEmptyList(),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createApplication(
            createBuiltin(Stdlib.ResolveArgs),
            createUnitList(createLambda(1, createInt(3))),
          ),
          createUnitList(
            createApplication(createBuiltin(Stdlib.Identity), createUnitList(createInt(4))),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createApplication(
            createBuiltin(Stdlib.ResolveArgs),
            createUnitList(createLambda(1, createVariable(0))),
          ),
          createUnitList(
            createApplication(createBuiltin(Stdlib.Identity), createUnitList(createInt(3))),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '3');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createApplication(
            createBuiltin(Stdlib.ResolveArgs),
            createUnitList(
              createLambda(
                3,
                createTriple(createVariable(2), createVariable(1), createVariable(0)),
              ),
            ),
          ),
          createTriple(
            createApplication(createBuiltin(Stdlib.Identity), createUnitList(createInt(3))),
            createApplication(createBuiltin(Stdlib.Identity), createUnitList(createInt(4))),
            createApplication(createBuiltin(Stdlib.Identity), createUnitList(createInt(5))),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '[3, 4, 5]');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};

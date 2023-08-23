// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_CollectSignal', (test) => {
    test('()', (assert, {
      createApplication,
      createEmptyList,
      createBuiltin,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectSignal),
          createEmptyList(),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{<InvalidFunctionArgsCondition:CollectSignal([])>}');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Signal)', (assert, {
      createApplication,
      createBuiltin,
      createCompositeSignal,
      createErrorCondition,
      createSignal,
      createString,
      createUnitList,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectSignal),
          createUnitList(createSignal(createErrorCondition(createString('foo')))),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{<ErrorCondition:"foo">}');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectSignal),
          createUnitList(
            createApplication(createBuiltin(Stdlib.Raise), createUnitList(createString('foo'))),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{<ErrorCondition:"foo">}');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectSignal),
          createUnitList(
            createCompositeSignal([
              createErrorCondition(createString('foo')),
              createErrorCondition(createString('bar')),
              createErrorCondition(createString('baz')),
            ]),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<ErrorCondition:"foo">,<ErrorCondition:"bar">,<ErrorCondition:"baz">}',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Signal, Signal)', (assert, {
      createApplication,
      createBuiltin,
      createCompositeSignal,
      createErrorCondition,
      createPair,
      createSignal,
      createString,
      createUnitList,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectSignal),
          createPair(
            createSignal(createErrorCondition(createString('foo'))),
            createSignal(createErrorCondition(createString('bar'))),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{<ErrorCondition:"foo">,<ErrorCondition:"bar">}');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectSignal),
          createPair(
            createApplication(createBuiltin(Stdlib.Raise), createUnitList(createString('foo'))),
            createApplication(createBuiltin(Stdlib.Raise), createUnitList(createString('bar'))),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '{<ErrorCondition:"foo">,<ErrorCondition:"bar">}');
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectSignal),
          createPair(
            createCompositeSignal([
              createErrorCondition(createString('foo')),
              createErrorCondition(createString('bar')),
            ]),
            createSignal(createErrorCondition(createString('baz'))),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<ErrorCondition:"foo">,<ErrorCondition:"bar">,<ErrorCondition:"baz">}',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectSignal),
          createPair(
            createSignal(createErrorCondition(createString('foo'))),
            createCompositeSignal([
              createErrorCondition(createString('bar')),
              createErrorCondition(createString('baz')),
            ]),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<ErrorCondition:"foo">,<ErrorCondition:"bar">,<ErrorCondition:"baz">}',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectSignal),
          createPair(
            createCompositeSignal([
              createErrorCondition(createString('foo')),
              createErrorCondition(createString('bar')),
            ]),
            createCompositeSignal([
              createErrorCondition(createString('baz')),
              createErrorCondition(createString('qux')),
            ]),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<ErrorCondition:"foo">,<ErrorCondition:"bar">,<ErrorCondition:"baz">,<ErrorCondition:"qux">}',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });

    test('(Signal, Signal, Signal)', (assert, {
      createApplication,
      createBuiltin,
      createCompositeSignal,
      createErrorCondition,
      createSignal,
      createString,
      createTriple,
      createUnitList,
      evaluate,
      format,
      getStateDependencies,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectSignal),
          createTriple(
            createSignal(createErrorCondition(createString('foo'))),
            createSignal(createErrorCondition(createString('bar'))),
            createSignal(createErrorCondition(createString('baz'))),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<ErrorCondition:"foo">,<ErrorCondition:"bar">,<ErrorCondition:"baz">}',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectSignal),
          createTriple(
            createApplication(createBuiltin(Stdlib.Raise), createUnitList(createString('foo'))),
            createApplication(createBuiltin(Stdlib.Raise), createUnitList(createString('bar'))),
            createApplication(createBuiltin(Stdlib.Raise), createUnitList(createString('baz'))),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<ErrorCondition:"foo">,<ErrorCondition:"bar">,<ErrorCondition:"baz">}',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.CollectSignal),
          createTriple(
            createCompositeSignal([
              createErrorCondition(createString('one')),
              createErrorCondition(createString('two')),
              createErrorCondition(createString('three')),
            ]),
            createCompositeSignal([
              createErrorCondition(createString('four')),
              createErrorCondition(createString('five')),
              createErrorCondition(createString('six')),
            ]),
            createCompositeSignal([
              createErrorCondition(createString('seven')),
              createErrorCondition(createString('eight')),
              createErrorCondition(createString('nine')),
            ]),
          ),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(
          format(result),
          '{<ErrorCondition:"one">,<ErrorCondition:"two">,<ErrorCondition:"three">,<ErrorCondition:"four">,<ErrorCondition:"five">,<ErrorCondition:"six">,<ErrorCondition:"seven">,<ErrorCondition:"eight">,<ErrorCondition:"nine">}',
        );
        assert.deepEqual(getStateDependencies(dependencies), []);
      })();
    });
  });
};

// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_Replace', (test) => {
    test('(String, String, String)', (assert, {
      createApplication,
      createBuiltin,
      createString,
      createTriple,
      evaluate,
      format,
      NULL,
      Stdlib,
    }) => {
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Replace),
          createTriple(createString(''), createString(''), createString('')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '""');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Replace),
          createTriple(createString('foo'), createString(''), createString('')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"foo"');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Replace),
          createTriple(createString('foo'), createString(''), createString('bar')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"barfoo"');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Replace),
          createTriple(createString('foo'), createString('foo'), createString('')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '""');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Replace),
          createTriple(createString('foo'), createString('bar'), createString('')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"foo"');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Replace),
          createTriple(createString('foobarbaz'), createString('bar'), createString('qux')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"fooquxbaz"');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Replace),
          createTriple(createString('foobarbaz'), createString('bar'), createString('quux')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"fooquuxbaz"');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Replace),
          createTriple(createString('foobarbaz'), createString('bar'), createString('qx')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"fooqxbaz"');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
      (() => {
        const expression = createApplication(
          createBuiltin(Stdlib.Replace),
          createTriple(createString('foofoofoobarbarbarbazbazbaz'), createString('barbarbar'), createString('qux')),
        );
        const [result, dependencies] = evaluate(expression, NULL);
        assert.strictEqual(format(result), '"foofoofooquxbazbazbaz"');
        assert.strictEqual(format(dependencies), 'NULL');
      })();
    });
  });
};

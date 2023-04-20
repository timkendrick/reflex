// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default (describe) => {
  describe('Stdlib_ParseDate', (test) => {
    test('(Timestamp)', (assert, {
      createApplication,
      createBuiltin,
      createTimestamp,
      createUnitList,
      evaluate,
      format,
      isTimestamp,
      NULL,
      Stdlib,
    }) => {
      const timestamp = Date.now();
      const expression = createApplication(
        createBuiltin(Stdlib.ParseDate),
        createUnitList(createTimestamp(timestamp)),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.ok(isTimestamp(result));
      assert.strictEqual(format(result), `Timestamp(${new Date(timestamp).toISOString()})`);
      assert.strictEqual(format(dependencies), 'NULL');
    });

    test('(Int)', (assert, {
      createApplication,
      createBuiltin,
      createInt,
      createUnitList,
      evaluate,
      format,
      isTimestamp,
      NULL,
      Stdlib,
    }) => {
      // TODO: increase integer value size to 64 bits to allow sensible timestamp ranges
      const timestamp = new Date(0x7fffffff);
      const expression = createApplication(
        createBuiltin(Stdlib.ParseDate),
        createUnitList(createInt(timestamp)),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.ok(isTimestamp(result));
      assert.strictEqual(format(result), `Timestamp(${new Date(timestamp).toISOString()})`);
      assert.strictEqual(format(dependencies), 'NULL');
    });

    test('(Float)', (assert, {
      createApplication,
      createBuiltin,
      createFloat,
      createUnitList,
      evaluate,
      format,
      isTimestamp,
      NULL,
      Stdlib,
    }) => {
      const timestamp = Date.now();
      const expression = createApplication(
        createBuiltin(Stdlib.ParseDate),
        createUnitList(createFloat(timestamp)),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.ok(isTimestamp(result));
      assert.strictEqual(format(result), `Timestamp(${new Date(timestamp).toISOString()})`);
      assert.strictEqual(format(dependencies), 'NULL');
    });

    test('(String)', (assert, {
      createApplication,
      createBuiltin,
      createString,
      createUnitList,
      evaluate,
      format,
      isTimestamp,
      NULL,
      Stdlib,
    }) => {
      const timestamp = Date.now();
      const expression = createApplication(
        createBuiltin(Stdlib.ParseDate),
        createUnitList(createString(new Date(timestamp).toISOString())),
      );
      const [result, dependencies] = evaluate(expression, NULL);
      assert.ok(isTimestamp(result));
      assert.strictEqual(format(result), `Timestamp(${new Date(timestamp).toISOString()})`);
      assert.strictEqual(format(dependencies), 'NULL');
    });
  });
};

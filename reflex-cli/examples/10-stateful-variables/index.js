import { hash, sequence } from 'reflex::core';
import { get, set, increment } from 'reflex::state';

const getCounter = (id) => {
  // Create a unique state variable ID for this counter
  // (note that the actual values passed to the hash function are unimportant,
  // all that matters is that the combination of arguments is unique)
  const uid = hash('counter', id);
  const initialValue = parseInt(0);
  return {
    value: () => get(uid, initialValue),
    increment: (token) => increment(uid, token),
    reset: (value, token) => set(uid, parseInt(value), token),
  };
};

const counter = getCounter('foo');

// Create idempotency tokens for performing one-off operations
// (the actual values passed to the hash function are unimportant,
// it only matters that the combination of arguments is unique)
const token1 = hash('operation', 1);
const token2 = hash('operation', 2);

const value = sequence(counter.reset(3, token1), (_result1) =>
  sequence(counter.increment(token2), (_result2) => counter.value()),
);

export default `Final value: ${value}`;

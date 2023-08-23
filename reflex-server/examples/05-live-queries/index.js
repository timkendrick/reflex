import { Resolver } from 'reflex::graphql';
import { now } from 'reflex::time';

// The same graph root will be used for both query and subscription operation roots
const root = {
  now: () => {
    // Current timestamp (sampled every 1000 milliseconds)
    const timestamp = now({ interval: 1000 });
    const millis = timestamp.getTime();
    return `Current UNIX time: ${Math.floor(millis / 1000)}`;
  },
  millis: () => {
    // Current timestamp (sampled every 1000 milliseconds)
    const timestamp = now({ interval: 1000 });
    return timestamp.getTime();
  },
  sampled: ({ interval }) => {
    // Emits a new result every `interval` milliseconds
    const sampled = now({ interval });
    const millis = sampled.getTime();
    // Graph roots can be arbitrarily complex/nested for all operation types
    return {
      millis,
      seconds: millis / 1000,
      labeled: ({ prefix }) => ({
        millis: `${prefix}: ${millis}`,
        seconds: `${prefix}: ${millis / 1000}`,
      }),
    };
  },
};

export default new Resolver({
  query: root,
  mutation: null,
  subscription: root,
});

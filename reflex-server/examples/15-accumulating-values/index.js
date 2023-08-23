import { Resolver } from 'reflex::graphql';
import { scan } from 'reflex::state';
import { now } from 'reflex::time';

export default new Resolver({
  query: null,
  mutation: null,
  subscription: {
    buckets: () => {
      // Calculate the distribution of the final digits of each emitted timestamp
      const buckets = scan(
        // Input expression: current timestamp in milliseconds (sampled every 1000 milliseconds)
        () => now({ interval: 1000 }),
        // Seed value
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        // Reducer function
        (state, timestamp) => {
          const buckets = state;
          const bucket = timestamp.getTime() % 10;
          return [
            ...buckets.slice(0, bucket),
            buckets[bucket] + 1,
            ...buckets.slice(bucket + 1, buckets.length),
          ];
        },
      );
      return buckets;
    },
  },
});

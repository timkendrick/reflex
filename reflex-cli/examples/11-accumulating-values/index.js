import { scan } from 'reflex::state';
import { now } from 'reflex::time';

// Calculate the distribution of the final digits of each emitted timestamp
const buckets = scan(
  // Input expression: current timestamp (sampled every 1000 milliseconds)
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

export default `Timestamp digit counts: ${buckets.join(', ')}`;

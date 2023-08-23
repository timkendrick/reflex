import { Resolver } from 'reflex::graphql';
import { now } from 'reflex::time';

export default new Resolver({
  query: null,
  mutation: null,
  subscription: {
    now: () => {
      // Current timestamp (sampled every 1000 milliseconds)
      const timestamp = now({ interval: 1000 });
      const millis = timestamp.getTime();
      return `Current UNIX time: ${Math.floor(millis / 1000)}`;
    },
  },
});

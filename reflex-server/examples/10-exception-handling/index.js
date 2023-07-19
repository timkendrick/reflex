import { Resolver } from 'reflex::graphql';
import { fetch } from 'reflex::http';

export default new Resolver({
  query: {
    message: () => {
      try {
        return fetch('http://@@@').json();
      } catch (error) {
        return error.message;
      }
    },
  },
  mutation: null,
  subscription: null,
});

import { Resolver } from 'reflex::graphql';
import { fetch } from 'reflex::http';

const fetchText = (url) => {
  try {
    return fetch(url).text();
  } catch (error) {
    return `Failed to load remote data (${error.message})`;
  }
};

export default new Resolver({
  query: {
    message: () => {
      return fetchText('http://@@@/message.txt');
    },
  },
  mutation: null,
  subscription: null,
});

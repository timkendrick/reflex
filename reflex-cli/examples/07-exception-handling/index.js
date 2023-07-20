import { fetch } from 'reflex::http';

const fetchText = (url) => {
  try {
    return fetch(url).text();
  } catch (error) {
    return `Failed to load remote data (${error.message})`;
  }
};

export default fetchText('http://@@@/message.txt');

import { ifPending } from 'reflex::core';
import { fetch } from 'reflex::http';

export default ifPending(
  () => {
    const user = fetch('https://jsonplaceholder.typicode.com/users/1').json();
    return `Hello, ${user.name}!`;
  },
  () => 'Loading...',
);

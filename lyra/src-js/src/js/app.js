import React, { useEffect, useState } from 'react';
import { test } from './events/event';
import css from './app.css';

const EVENTS = {
  0: 'ping',
  1: 'break',
  2: 'data',
};
const EVENT_ARGS = {
  0: null,
  1: null,
  2: { msg: 'hello' },
};

function App(props) {
  const [value, setValue] = useState('Nil');
  const [inc, setInc] = useState(0);

  useEffect(() => {
    if (inc >= 3) {
      setInc(0);
    }
  }, [inc]);

  const onClick = () => {
    rpc
      .call(EVENTS[inc], EVENT_ARGS[inc])
      .then((r) => {
        console.log(r);
        setInc(inc + 1);
        setValue(r);
      })
      .catch((e) => {
        console.log(e);
        setInc(inc + 1);
      });
  };

  return (
    <>
      <button onClick={onClick}>Send RPC</button>
      <div className={css.app}>{value}</div>
    </>
  );
}

export { App };

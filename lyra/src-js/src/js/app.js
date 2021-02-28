import React, { useState } from 'react';
import { test } from './events/event';
import css from './app.css';

function App(props) {
  const value = test('hello');
  return <div className={css.app}>{value}</div>;
}

export { App };

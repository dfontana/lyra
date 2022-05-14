import React from 'react';
import css from './app.css';

const className = (s) => `${css.searchResult} ${s ? css.selected : ''}`;

function SearchResult({ id, value, selected, ...rest }) {
  return (
    <div className={className(selected)} {...rest}>
      {id}: {value}
    </div>
  );
}

export default SearchResult;

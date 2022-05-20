import React from 'react';
import './app.css';

const className = (s) => `searchResult ${s ? 'selected' : ''}`;

function SearchResult({ id, value, icon, selected, ...rest }) {
  return (
    <div className={className(selected)} {...rest}>
      <img className="searchIcon" src={icon} alt="" />
      <span>
        {id}: {value}
      </span>
    </div>
  );
}

export default SearchResult;

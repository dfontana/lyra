import React from 'react';
import { Avatar } from '@geist-ui/core';
import './search.css';

const className = (s) => `searchResult ${s ? 'selected' : ''}`;

function SearchResult({ type, value, icon, selected, ...rest }) {
  return (
    <div className={className(selected)} {...rest}>
      {type !== 'WebQuery' && (
        <span className="searchResult-icon">
          <Avatar src={icon} alt="" />
        </span>
      )}
      <span className="searchResult-label">{value}</span>
    </div>
  );
}

export default SearchResult;

import React from 'react';
import { Avatar } from '@geist-ui/core';
import './app.css';

const className = (s) => `searchResult ${s ? 'selected' : ''}`;

function SearchResult({ value, icon, selected, ...rest }) {
  return (
    <div className={className(selected)} {...rest}>
      <span className="searchResult-icon">
        <Avatar src={icon} alt="" />
      </span>
      <span className="searchResult-label">{value}</span>
    </div>
  );
}

export default SearchResult;

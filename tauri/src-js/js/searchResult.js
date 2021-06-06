import React from "react";
import css from "./app.css";

const className = (s) => `${css.searchResult} ${s ? css.selected : ""}`;

function SearchResult({ id, value, selected }) {
  return (
    <div className={className(selected)}>
      {id}: {value}
    </div>
  );
}

export default SearchResult;

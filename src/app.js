import React, { useCallback, useEffect, useState, useRef } from 'react';
import SearchResult from './searchResult';
import useKeyPress from './useKeyPress';
import './app.css';

const CALL = 'call';
const EVENTS = {
  SUBMIT: 'Submit',
  SEARCH: 'Search',
};


function App() {
  // TODO onKeyPress for the input.
  //      Should handle "escape" to close the window too (RPC shutdown)
  const searchRef = useRef();
  const [search, setSearch] = useState('');
  const [selection, setSelected] = useState(0);
  const [results, setResults] = useState([]);

  const isArrowDown = useKeyPress('ArrowDown', searchRef);
  const isArrowUp = useKeyPress('ArrowUp', searchRef);
  const isEnter = useKeyPress('Enter', searchRef);

  useEffect(() => {
    if (isArrowDown && selection < results.length - 1) {
      setSelected(selection + 1);
    }
  }, [isArrowDown, selection, setSelected]);

  useEffect(() => {
    if (isArrowUp && selection > 0) {
      setSelected(selection - 1);
    }
  }, [isArrowUp, selection, setSelected]);

  useEffect(() => {
    if (isEnter) {
      // rpc.call(CALL, { type: EVENTS.SUBMIT, value: selection }).catch(console.error);
    }
  }, [isEnter, selection]);
  const onKeyPress = ({ key }) => {
    switch (key) {
      case 'Enter':
      case 'ArrowDown':
      case 'ArrowUp':
        return;
      default:
        setSelected(0);
      // rpc
      //   .call(CALL, { type: EVENTS.SEARCH, value: search })
      //   .then(setResults)
      //   .catch(console.error);
    }
  };

  const onChange = useCallback((event) => setSearch(event.target.value), [setSearch]);


  return (
    <div className="searchRoot">
      <input
        ref={searchRef}
        className="searchInput"
        type="text"
        autofocus
        autoCorrect="off"
        onKeyPress={onKeyPress}
        onChange={onChange}
        value={search}
      />
      {results.map(({ id, value }) => (
        <SearchResult key={id} id={id} value={value} selected={id == selection} />
      ))}
    </div>
  );
}

export default App;

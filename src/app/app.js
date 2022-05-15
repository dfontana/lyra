import React, { useCallback, useEffect, useState, useRef } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import SearchResult from './searchResult';
import useKeyPress, { useKeyPressResetable } from './useKeyPress';
import './app.css';

const { RESET } = window.__LYRA__.events;
const { SEARCH, SUBMIT, CLOSE } = window.__LYRA__.calls;
const { INPUT_HEIGHT, OPTION_HEIGHT, FONT_SIZE } = window.__LYRA__.styles;

function App() {
  const [search, setSearch] = useState('');
  const [selection, setSelected] = useState(0);
  const [results, setResults] = useState([]);

  const searchRef = useRef();
  const isArrowDown = useKeyPress('ArrowDown', searchRef);
  const isArrowUp = useKeyPress('ArrowUp', searchRef);
  const isEnter = useKeyPress('Enter', searchRef);
  const [isEscape, resetEscape] = useKeyPressResetable('Escape', searchRef);

  useEffect(() => {
    let unlisten = null;
    listen(RESET, () => {
      setSearch('');
      setSelected(0);
      setResults([]);
    }).then((func) => {
      unlisten = func;
    });
    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [setSearch, setSelected, setResults]);

  useEffect(() => {
    if (isArrowDown && selection < results.length - 1) {
      setSelected(selection + 1);
    }
  }, [isArrowDown, selection, setSelected, results]);

  useEffect(() => {
    if (isArrowUp && selection > 0) {
      setSelected(selection - 1);
    }
  }, [isArrowUp, selection, setSelected]);

  useEffect(() => {
    if (isEnter) {
      invoke(SUBMIT, { selection }).catch(console.error);
    }
  }, [isEnter, selection]);

  useEffect(() => {
    if (isEscape) {
      resetEscape();
      invoke(CLOSE).catch(console.error);
    }
  }, [isEscape, resetEscape]);

  const onKeyPress = ({ key }) => {
    switch (key) {
      case 'Enter':
      case 'ArrowDown':
      case 'ArrowUp':
        return;
      default:
        // setSelected(0);
        invoke(SEARCH, { search }).then(setResults).catch(console.error);
    }
  };

  const onChange = useCallback((event) => setSearch(event.target.value), [setSearch]);

  return (
    <div className="searchRoot">
      <input
        ref={searchRef}
        className="searchInput"
        type="text"
        autoFocus
        autoCorrect="off"
        onKeyPress={onKeyPress}
        onChange={onChange}
        value={search}
        style={{
          height: `${INPUT_HEIGHT}px`,
          fontSize: `${FONT_SIZE}px`,
        }}
      />
      {results.map(({ id, value }) => (
        <SearchResult
          key={id}
          id={id}
          value={value}
          selected={id === selection}
          style={{
            fontSize: `${FONT_SIZE}px`,
            height: `${OPTION_HEIGHT}px`,
          }}
        />
      ))}
    </div>
  );
}

export default App;

import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { useCallback, useEffect, useState } from 'react';
import useNavigation from './useNavigation';
import SearchResult from './searchResult';
import './app.css';

const { RESET } = window.__LYRA__.events;
const { SEARCH, SUBMIT, CLOSE } = window.__LYRA__.calls;
const { INPUT_HEIGHT, OPTION_HEIGHT, FONT_SIZE } = window.__LYRA__.styles;

function App() {
  const [search, setSearch] = useState('');
  const [results, setResults] = useState([]);
  const [selection, resetNav] = useNavigation({
    results,
    onSubmit: (selection) => invoke(SUBMIT, { selected: results[selection] }).catch(console.error),
    onClose: () => invoke(CLOSE).catch(console.error),
  });

  useEffect(() => {
    let unlisten = null;
    listen(RESET, () => {
      setSearch('');
      setResults([]);
      resetNav();
    }).then((func) => {
      unlisten = func;
    });
    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [setSearch, setResults, resetNav]);

  const onChange = useCallback((e) => setSearch(e.target.value), [setSearch]);
  const onMouseDown = useCallback((e) => e.preventDefault(), []);
  const triggerSearch = useCallback(
    ({ key }) => {
      switch (key) {
        case 'Enter':
        case 'ArrowDown':
        case 'ArrowUp':
          return;
        default:
          invoke(SEARCH, { search }).then(setResults).catch(console.error);
      }
    },
    [search]
  );

  return (
    <div className="searchRoot" onMouseDown={onMouseDown}>
      <input
        className="searchInput"
        type="text"
        autoFocus
        autoCorrect="off"
        onChange={onChange}
        onKeyUp={triggerSearch}
        value={search}
        style={{
          height: `${INPUT_HEIGHT}px`,
          fontSize: `${FONT_SIZE}px`,
        }}
      />
      {results.map(({ label, icon }, idx) => (
        <SearchResult
          key={label}
          id={label}
          value={label}
          icon={icon}
          selected={idx === selection}
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

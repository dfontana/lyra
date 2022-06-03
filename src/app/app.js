import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { useCallback, useEffect, useState } from 'react';
import useNavigation from './useNavigation';
import SearchResult from './searchResult';
import './app.css';

const { RESET } = window.__LYRA__.events;
const { SEARCH, SELECT_SEARCH, SUBMIT, CLOSE } = window.__LYRA__.calls;
const { INPUT_HEIGHT, OPTION_HEIGHT, FONT_SIZE } = window.__LYRA__.styles;

const isSearcherSelected = (results, selection, search) => {
  const item = results[selection];
  return item?.type === 'Searcher' && search.startsWith(item?.shortname) && search.includes(' ');
};

const split = (str, sep, n) => {
  let split = str.split(sep);
  if (split.length <= n) return split;
  var out = split.slice(0, n - 1);
  out.push(split.slice(n - 1).join(sep));
  return out;
};

function App() {
  const [search, setSearch] = useState('');
  const [results, setResults] = useState([]);
  const [selection, resetNav] = useNavigation({
    results,
    onSubmit: (selection) => {
      let selected = { ...results[selection] };

      const isSearcher = isSearcherSelected(results, selection, search);
      if (isSearcher) {
        const expectArgs = results[selection].required_args;
        const args = split(search, ' ', expectArgs + 1);
        console.log(args, expectArgs);
        if (args.length - 1 !== expectArgs) {
          // Not yet ready to search need more args
          return;
        }
        // ready to search, add the args in
        selected.args = args.slice(1);
      }
      invoke(SUBMIT, { selected }).catch(console.error);
    },
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

  useEffect(() => {
    if (results.length > 1 && isSearcherSelected(results, selection, search)) {
      // Clear results to only be the selected item
      setResults(results.filter((_, i) => i === selection));
      resetNav();
      invoke(SELECT_SEARCH).catch(console.error);
      return;
    }
  }, [selection, results, search, resetNav, setResults]);

  const onChange = useCallback((e) => setSearch(e.target.value), [setSearch]);
  const onMouseDown = useCallback((e) => e.preventDefault(), []);
  const triggerSearch = useCallback(
    ({ key }) => {
      if (isSearcherSelected(results, selection, search)) {
        // Do not trigger a search when a searcher is selected and a space has been entered
        return;
      }
      switch (key) {
        case 'Enter':
        case 'ArrowDown':
        case 'ArrowUp':
          return;
        default:
          invoke(SEARCH, { search }).then(setResults).catch(console.error);
      }
    },
    [search, selection, results]
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

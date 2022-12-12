import { invoke } from '@tauri-apps/api/tauri';
import { useCallback, useEffect, useState } from 'react';
import useNavigation from '../useNavigation';
import SearchResult from './searchResult';
import './search.css';

const { SEARCH, SELECT_SEARCH, SUBMIT } = window.__LYRA__.calls;
const { OPTION_HEIGHT, FONT_SIZE } = window.__LYRA__.styles;

const isSearcherSelected = (results, selection, search) => {
  const item = results[selection];
  return item?.type === 'Searcher' && search.startsWith(item?.shortname) && search.includes(' ');
};

const isWebQuerySelected = (results, selection) => {
  return results[selection]?.type === 'WebQuery';
};

const split = (str, sep, n) => {
  let split = str.split(sep);
  if (split.length <= n) return split;
  var out = split.slice(0, n - 1);
  out.push(split.slice(n - 1).join(sep));
  return out;
};

function Search({ inputRef, resetRef, search }) {
  const [results, setResults] = useState([]);

  const [selection, resetNav] = useNavigation({
    results,
    onSubmit: (selection) => {
      let selected = { ...results[selection] };

      if (isSearcherSelected(results, selection, search)) {
        const expectArgs = results[selection].required_args;
        const args = split(search, ' ', expectArgs + 1);
        console.log(args, expectArgs);
        if (args.length - 1 !== expectArgs) {
          // Not yet ready to search need more args
          return;
        }
        // ready to search, add the args in
        selected.args = args.slice(1);
      } else if (isWebQuerySelected(results, selection)) {
        selected.query = search;
      }
      invoke(SUBMIT, { selected }).catch(console.error);
    },
  });

  useEffect(() => {
    if (results.length > 1 && isSearcherSelected(results, selection, search)) {
      // Clear results to only be the selected item
      setResults(results.filter((_, i) => i === selection));
      resetNav();
      invoke(SELECT_SEARCH).catch(console.error);
      return;
    }
  }, [selection, results, search, resetNav, setResults]);

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

  useEffect(() => {
    resetRef.current = () => {
      resetNav();
      setResults([]);
    };
    return () => {
      resetRef.current = () => {};
    }
  }, [resetRef, resetNav, setResults]);

  useEffect(() => {
    let node = inputRef.current;
    node.onkeyup = triggerSearch;
    return () => {
      node.onkeyup = null; 
    };
  }, [inputRef, triggerSearch]);

  return (
    <>
      {results.map(({ type, label, icon }, idx) => (
        <SearchResult
          key={label}
          type={type}
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
    </>
  )
}

export default Search;

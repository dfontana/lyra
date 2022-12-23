import { invoke } from '@tauri-apps/api/tauri';
import { useCallback, useEffect, useState } from 'react';
import useNavigation from '../useNavigation';
import SearchResult from './searchResult';

const { SEARCH, SELECT_SEARCH, SUBMIT } = window.__LYRA__.calls;
const { OPTION_HEIGHT, FONT_SIZE } = window.__LYRA__.styles;

const isSearcherSelected = (selected) => {
  return selected?.type === 'Searcher';
};

const isSearcherSelectedWhileTemplating = (item, search) => {
  return isSearcherSelected(item) && search.startsWith(item?.shortname) && search.includes(' ');
};

const isSearcherNotSelectedWhenTemplateStarts = (results, selection, search) => {
  let selected = results[selection];
  return (
    selected?.type !== 'Searcher' &&
    search.endsWith(' ') &&
    search.startsWith(results[0]?.shortname)
  );
};

const isWebQuerySelected = (selected) => {
  return selected?.type === 'WebQuery';
};

const split = (str, sep, n) => {
  let split = str.trim().split(sep);
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

      if (isSearcherSelected(selected)) {
        const expectArgs = results[selection].required_args;
        const args = split(search, ' ', expectArgs + 1);
        if (args.length - 1 !== expectArgs) {
          // Not yet ready to search need more args
          // TODO: We don't have access to setSearch so I can't force a space
          //       if there isn't one yet. Ideally we can do this or just better model
          //       how this renders in the UI (eg on enter we get prompted or something)
          return;
        }
        // ready to search, add the args in
        selected.args = args.slice(1);
      } else if (isWebQuerySelected(selected)) {
        selected.query = search;
      }
      invoke(SUBMIT, { selected }).catch(console.error);
    },
  });

  useEffect(() => {
    if (results.length <= 1) {
      return;
    }
    if (isSearcherSelectedWhileTemplating(results[selection], search)) {
      // Clear results to only be the selected item
      setResults(results.filter((_, i) => i === selection));
      resetNav();
      invoke(SELECT_SEARCH).catch(console.error);
      return;
    } else if (isSearcherNotSelectedWhenTemplateStarts(results, selection, search)) {
      // Change selection to only be the searcher. This _may_ be a bug in waiting
      // as it assumes the matching item is the first in the results.
      setResults(results.filter((sh, _) => sh?.shortname === search.trim()));
      resetNav();
      invoke(SELECT_SEARCH).catch(console.error);
      return;
    }
  }, [selection, results, search, resetNav, setResults]);

  const triggerSearch = useCallback(
    ({ key }) => {
      if (isSearcherSelectedWhileTemplating(results[selection], search)) {
        // Do not trigger a search when a searcher is selected and a space has been entered
        return;
      }
      switch (key) {
        case 'Enter':
        case 'ArrowDown':
        case 'ArrowUp':
          return;
        default:
          invoke(SEARCH, { search: search.trim() }).then(setResults).catch(console.error);
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
    };
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
  );
}

export default Search;

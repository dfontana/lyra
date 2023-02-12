import { invoke } from '@tauri-apps/api/tauri';
import { useCallback, useEffect, useState } from 'react';
import useNavigation from './useNavigation';
import useWindowResize from './useWindowSize';
import CalcResult from './calcResult';
import SearchResult from './searchResult';
const { SEARCH, SUBMIT } = window.__LYRA__.calls;

const TEMPLATE_NOT_STARTED = 'not_start';
const TEMPLATE_STARTED = 'start';
const TEMPLATE_COMPLETED = 'done';

const isDefaultSearch = (pluginValue) => pluginValue.shortname === '';
const isTemplatingStarted = (maybePair, search) => {
  if (!maybePair) {
    // Can't be templating nothing
    return false;
  }
  let [_, pluginValue] = maybePair;
  return (
    !isDefaultSearch(pluginValue) &&
    search.startsWith(pluginValue.shortname) &&
    search.includes(' ')
  );
};
const isTemplatingComplete = (maybePair, search) => {
  if (!maybePair) {
    // Can't be templating nothing
    return false;
  }
  let [_, pluginValue] = maybePair;
  const args = extractArgs(pluginValue, search);
  return args.length === pluginValue.required_args;
};

const extractArgs = (pluginValue, search) => {
  const expectArgs = pluginValue.required_args;
  const args = split(search, ' ', expectArgs + 1);
  const isDefault = isDefaultSearch(pluginValue);
  return isDefault && search !== '' ? [search] : args.slice(1);
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
  useWindowResize(results);
  const [templateState, setTemplateState] = useState(TEMPLATE_NOT_STARTED);

  const [selection, resetNav] = useNavigation({
    results,
    onSubmit: (selection) => {
      let isCalc = results?.[selection]?.[0] === 'calc';
      let pair = results?.[selection];
      let selected = null;
      if (isCalc) {
        selected = pair[1].Ok;
      } else if (templateState === TEMPLATE_COMPLETED) {
        let args = extractArgs(pair[1], search);
        selected = { ...pair[1], args };
      } else {
        // TODO: UI - We don't have access to setSearch so I can't force a space
        //       if there isn't one yet. Ideally we can do this or just better model
        //       how this renders in the UI (eg on enter we get prompted or something)
        return;
      }

      invoke(SUBMIT, { forPlugin: pair[0], selected }).catch(console.error);
    },
  });

  useEffect(() => {
    switch (templateState) {
      case TEMPLATE_NOT_STARTED:
        // Check if we need enter templating state
        if (isTemplatingStarted(results?.[selection], search)) {
          // Set our focus to what we are working on
          setResults(results.filter((_, i) => i === selection));
          setTemplateState(TEMPLATE_STARTED);
          resetNav();
        } else if (isTemplatingComplete(results?.[selection], search)) {
          // The default searcher can make this jump
          setTemplateState(TEMPLATE_COMPLETED);
        }
        break;
      case TEMPLATE_STARTED:
        if (!isTemplatingStarted(results?.[selection], search)) {
          setTemplateState(TEMPLATE_NOT_STARTED);
        } else if (isTemplatingComplete(results?.[selection], search)) {
          setTemplateState(TEMPLATE_COMPLETED);
        }
        break;
      case TEMPLATE_COMPLETED:
        if (!isTemplatingComplete(results?.[selection], search)) {
          setTemplateState(TEMPLATE_STARTED);
        }
        break;
      default:
      // Not reachable
    }
  }, [selection, results, search, resetNav, setResults, templateState, setTemplateState]);

  const triggerSearch = useCallback(
    ({ key }) => {
      if (templateState !== TEMPLATE_NOT_STARTED) {
        // Don't search while templating
        return;
      }
      switch (key) {
        case 'Enter':
        case 'ArrowDown':
        case 'ArrowUp':
          return;
        default:
          invoke(SEARCH, { search: search.trim() })
            .then((r) => {
              resetNav();
              setResults(r);
            })
            .catch(console.error);
      }
    },
    [search, resetNav, templateState]
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
      {results.map(([pl, v], idx) => {
        switch (pl) {
          case 'webq':
            return (
              <SearchResult
                key={v.label}
                id={v.label}
                value={v.label}
                icon={v.icon}
                selected={idx === selection}
              />
            );
          case 'calc':
            return <CalcResult key="calc" result={v?.Ok} error={v?.Err} expression={search} />;
          default:
            return <></>;
        }
      })}
    </>
  );
}

export default Search;

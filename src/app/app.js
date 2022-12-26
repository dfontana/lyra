import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/tauri';
import { useCallback, useEffect, useRef, useState } from 'react';
import Search from './search/search';
import Calculator from './calc/calculator';
import { useKeyPressResetable } from './useKeyPress';
import useWindowResize from './useWindowSize';

const { RESET } = window.__LYRA__.events;
const { CLOSE } = window.__LYRA__.calls;

const MODES = {
  INIT: 'init',
  SEARCH: 'search',
  CALC: 'calc',
};

function App() {
  const [mode, setMode] = useState(MODES.INIT);
  const [initInput, setInitInput] = useState('');
  const [isEscape, resetEscape] = useKeyPressResetable('Escape');
  const resetSize = useWindowResize(null);
  const inputRef = useRef();
  const resetRef = useRef(() => {});

  // TODO: Known bug https://github.com/tauri-apps/tao/issues/434
  //       if window is resized while hidden it moves. You can try to find a way to solve this by resizing
  //       after it's un-hidden, but is proving tricky.
  useEffect(() => {
    // Ensure the window is the rgith starting size
    resetSize()
  }, [resetSize])

  useEffect(() => {
    if (isEscape) {
      resetEscape();
      invoke(CLOSE).catch(console.error);
    }
  }, [isEscape, resetEscape]);

  useEffect(() => {
    // TODO: UI = this should change to use the prefix system for plugins. You'll want
    //       to make sure `initInput` never has a prefix attaached to it when calling
    //       the backend, so design wisely. MODES may not be needed in the long run
    //       since we want to join calc & search functionality together into one UI page
    //       (but maybe with some kind of pluggable behavior since calc renders errors, etc)
    //       Calc, for example, prunes this before submission.
    if (initInput.startsWith('=')) {
      if (mode !== MODES.CALC) {
        setMode(MODES.CALC);
      }
    } else if (initInput && mode !== MODES.SEARCH) {
      setMode(MODES.SEARCH);
    } else if (!initInput && mode !== MODES.INIT) {
      // Ensure the window shrinks after leaving modes
      resetSize();
      setMode(MODES.INIT);
    }
  }, [mode, setMode, initInput, resetSize]);

  useEffect(() => {
    let unlisten = null;
    listen(RESET, () => {
      setMode(MODES.INIT);
      setInitInput('');
      resetEscape();
      resetRef.current();
    }).then((func) => {
      unlisten = func;
    });
    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [setMode, setInitInput, resetRef, resetEscape]);

  const onMouseDown = useCallback((e) => e.preventDefault(), []);
  const onChange = useCallback((e) => setInitInput(e.target.value), [setInitInput]);

  return (
    <div className="appRoot" onMouseDown={onMouseDown}>
      <input
        ref={inputRef}
        className="appInput"
        type="text"
        autoFocus
        autoCorrect="off"
        onChange={onChange}
        value={initInput}
      />
      {(() => {
        switch (mode) {
          case MODES.SEARCH:
            return <Search resetRef={resetRef} inputRef={inputRef} search={initInput} />;
          case MODES.CALC:
            return <Calculator resetRef={resetRef} inputRef={inputRef} expression={initInput} />;
          default:
            return;
        }
      })()}
    </div>
  );
}

export default App;

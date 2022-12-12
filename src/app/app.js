import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/tauri';
import { useCallback, useEffect, useRef, useState } from 'react';
import Search from './search/search';
import Calculator from './calc/calculator';
import { useKeyPressResetable } from './useKeyPress';
import './app.css';

const { RESET } = window.__LYRA__.events;
const { RESET_SIZE, CLOSE } = window.__LYRA__.calls;
const { INPUT_HEIGHT, FONT_SIZE } = window.__LYRA__.styles;

const MODES = {
  INIT: 'init',
  SEARCH: 'search',
  CALC: 'calc',
};

function App() {
  const [mode, setMode] = useState(MODES.INIT);
  const [initInput, setInitInput] = useState('');
  const [isEscape, resetEscape] = useKeyPressResetable('Escape');
  const inputRef = useRef();
  const resetRef = useRef(() => {});

  useEffect(() => {
    if (isEscape) {
      resetEscape();
      invoke(CLOSE).catch(console.error);
    }
  }, [isEscape, resetEscape]);

  useEffect(() => {
    if (initInput.startsWith('=')) {
      if (mode !== MODES.CALC) {
        setMode(MODES.CALC);
      }
    } else if (initInput && mode !== MODES.SEARCH) {
      setMode(MODES.SEARCH);
    } else if (!initInput && mode !== MODES.INIT) {
      invoke(RESET_SIZE, {}).catch(console.error);
      setMode(MODES.INIT);
    }
  }, [mode, setMode, initInput]);

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
        style={{
          height: `${INPUT_HEIGHT}px`,
          fontSize: `${FONT_SIZE}px`,
        }}
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

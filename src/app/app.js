import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/tauri';
import { useCallback, useEffect, useRef, useState } from 'react';
import Search from './search/search';
import useKeybinds from './useKeybinds';
import './app.css';

const { RESET } = window.__LYRA__.events;
const { RESET_SIZE } = window.__LYRA__.calls;
const { INPUT_HEIGHT, FONT_SIZE } = window.__LYRA__.styles;

const MODES = {
  INIT: 'init',
  SEARCH: 'search',
  CALC: 'calc',
};

function App() {
  const [mode, setMode] = useState(MODES.INIT);
  const [initInput, setInitInput] = useState('');
  const resetBinds = useKeybinds();
  const inputRef = useRef();
  const resetRef = useRef(() => { });

  useEffect(() => {
    if (initInput && mode !== MODES.SEARCH) {
      setMode(MODES.SEARCH);
    } else if (!initInput && mode !== MODES.INIT) {
      invoke(RESET_SIZE, { }).catch(console.error);
      setMode(MODES.INIT);
    }
  }, [mode, setMode, initInput]);

  useEffect(() => {
    let unlisten = null;
    listen(RESET, () => {
      setMode(MODES.INIT);
      setInitInput('');
      resetBinds();
      resetRef.current();
    }).then((func) => {
      unlisten = func;
    });
    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [setMode, setInitInput, resetRef, resetBinds]);

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
          case MODES.SEARCH: return (
            <Search resetRef={resetRef} inputRef={inputRef} search={initInput} />
          )
          case MODES.CALC: return (<div>meow</div>)
          default: return
        };
      })()}

    </div>
  );
}

export default App;

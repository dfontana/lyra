import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/tauri';
import { useCallback, useEffect, useRef, useState } from 'react';
import Search from './search';
import { useKeyPressResetable } from './useKeyPress';

const { RESET } = window.__LYRA__.events;
const { CLOSE } = window.__LYRA__.calls;

function App() {
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
    let unlisten = null;
    listen(RESET, () => {
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
  }, [setInitInput, resetRef, resetEscape]);

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
      <Search resetRef={resetRef} inputRef={inputRef} search={initInput} />
    </div>
  );
}

export default App;

import { invoke } from '@tauri-apps/api/tauri';
import { writeText } from '@tauri-apps/api/clipboard';
import { useCallback, useEffect, useState } from 'react';
import { useKeyPressResetable } from '../useKeyPress';
import useWindowResize from '../useWindowSize';

const { CLOSE, CALCULATE } = window.__LYRA__.calls;

// TODO: UI - its not yet clear where it may be needed, but the calc may have to reduce it's
//       window size to 1 search result like when a searcher is selected. The `select` call
//       is available for this.
export default function Calculator({ inputRef, resetRef, expression }) {
  const [result, setResult] = useState('');
  const [error, setError] = useState({ message: '', start: 0, end: 0 });
  const [isEnter, resetEnter] = useKeyPressResetable('Enter');
  useWindowResize(result);

  useEffect(() => {
    if (isEnter) {
      resetEnter();
      writeText(result).catch(console.error);
      invoke(CLOSE).catch(console.error);
    }
  }, [result, isEnter, resetEnter]);

  const triggerCalc = useCallback(
    ({ key }) => {
      switch (key) {
        case 'Enter':
        case 'ArrowLeft':
        case 'ArrowRight':
          return;
        default:
          // TODO: UI - this is changed to submit for calc plugin specifically
          invoke(CALCULATE, { expression: expression.slice(1) })
            .then(setResult)
            .catch((err) => {
              setResult('');
              setError(err);
            });
      }
    },
    [expression, setResult]
  );

  useEffect(() => {
    let node = inputRef.current;
    node.onkeyup = triggerCalc;
    return () => {
      node.onkeyup = null;
    };
  }, [inputRef, triggerCalc]);

  let body = [];
  if (result) {
    body = [{ v: result }];
  } else if (!error.start) {
    body = [{ v: error.message }];
  } else {
    body = [
      { v: expression.slice(1, error.start), cx: '' },
      { v: expression.slice(error.start, error.end + 1), cx: 'calcError' },
      { v: expression.slice(error.end + 1), cx: '' },
    ];
  }
  return (
    <div className="calcResult">
      {body.map((c, i) => (
        <span key={i} className={c.cx}>
          {c.v}
        </span>
      ))}
    </div>
  );
}

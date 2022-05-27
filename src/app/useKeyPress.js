import { useState, useEffect } from 'react';

export default function useKeyPress(targetKey) {
  return useInternal(targetKey)[0];
}

function useKeyPressResetable(targetKey) {
  return useInternal(targetKey);
}

export { useKeyPressResetable };

function useInternal(targetKey) {
  const [keyPressed, setKeyPressed] = useState(false);

  function reset() {
    setKeyPressed(false);
  }

  useEffect(() => {
    function downHandler(event) {
      if (event.key === targetKey) {
        setKeyPressed(true);
        event.preventDefault();
      }
    }

    function upHandler(event) {
      if (event.key === targetKey) {
        setKeyPressed(false);
        event.preventDefault();
      }
    }

    document.addEventListener('keydown', downHandler);
    document.addEventListener('keyup', upHandler);
    return () => {
      document.removeEventListener('keydown', downHandler);
      document.removeEventListener('keyup', upHandler);
    };
  }, [targetKey]);

  return [keyPressed, reset];
}

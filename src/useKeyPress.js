import { useState, useEffect } from 'react';

export default function useKeyPress(targetKey, ref) {
  return useInternal(targetKey, ref)[0];
}

function useKeyPressResetable(targetKey, ref) {
  return useInternal(targetKey, ref);
}

export { useKeyPressResetable }

function useInternal(targetKey, ref) {
  const [keyPressed, setKeyPressed] = useState(false);

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
  };

  function reset() {
    setKeyPressed(false);
  }

  useEffect(() => {
    ref.current.addEventListener('keydown', downHandler);
    ref.current.addEventListener('keyup', upHandler);

    return () => {
      ref.current.removeEventListener('keydown', downHandler);
      ref.current.removeEventListener('keyup', upHandler);
    };
  }, [upHandler, downHandler]);

  return [keyPressed, reset];
}

import { useState, useEffect } from 'react';

export default function useKeyPress(targetKey, ref) {
  return useInternal(targetKey, ref)[0];
}

function useKeyPressResetable(targetKey, ref) {
  return useInternal(targetKey, ref);
}

export { useKeyPressResetable };

function useInternal(targetKey, ref) {
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

    let node = ref.current;
    node.addEventListener('keydown', downHandler);
    node.addEventListener('keyup', upHandler);
    return () => {
      node.removeEventListener('keydown', downHandler);
      node.removeEventListener('keyup', upHandler);
    };
  }, [targetKey, ref]);

  return [keyPressed, reset];
}

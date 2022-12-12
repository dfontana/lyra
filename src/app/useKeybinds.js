import { invoke } from '@tauri-apps/api/tauri';
import { useEffect } from 'react';
import { useKeyPressResetable } from './useKeyPress';

const { CLOSE } = window.__LYRA__.calls;

export default function useKeybinds() {
  const [isEscape, resetEscape] = useKeyPressResetable('Escape');

  useEffect(() => {
    if (isEscape) {
      resetEscape();
      invoke(CLOSE).catch(console.error)
    }
  }, [isEscape, resetEscape]);

  return () => {
    resetEscape();
  };
}

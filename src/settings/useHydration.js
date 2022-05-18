import { invoke } from '@tauri-apps/api/tauri';
import { useState, useEffect } from 'react';

export default function useHydration(tauriCommand) {
  const [isLoading, setIsLoading] = useState(true);
  const [state, setState] = useState(null);

  useEffect(() => {
    invoke(tauriCommand).then((data) => {
      setState(data);
      setIsLoading(false);
    });
  }, [tauriCommand]);

  return [isLoading, state];
}

import { useCallback, useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { appWindow, LogicalSize } from '@tauri-apps/api/window';

export default function useWindowResize(trigger) {
  const [initialRect, setInitialRect] = useState({ width: 0, height: 0 });
  const [hasInit, setHasInit] = useState(false);
  const [resizeMe, setResizeMe] = useState(false);

  useEffect(() => {
    if (hasInit) {
      return;
    }
    let rect = document.body.getBoundingClientRect();
    if (rect.width !== 0 && rect.height !== 0) {
      setInitialRect(rect);
      setHasInit(true);
    }
  }, [hasInit, setHasInit, setInitialRect]);

  let resize = useCallback((rect) => {
    // TODO: (Part of bug described below, don't resize unless visible)
    appWindow.isVisible().then((isVis) => {
      if (isVis && rect.width !== 0 && rect.height !== 0) {
        appWindow.setSize(new LogicalSize(rect.width, rect.height));
      }
    });
  }, []);

  useEffect(() => {
    if (trigger !== null) {
      resize(document.body.getBoundingClientRect());
    }
  }, [trigger, resize]);

  let resetSize = useCallback(() => resize(initialRect), [initialRect, resize]);

  // TODO: Known bug https://github.com/tauri-apps/tao/issues/434
  //       For now the workaround is these two effects + the event. When the window is toggled
  //       visible again we'll resize it, which causes a brief visual defect but the best we can do.
  useEffect(() => {
    if (resizeMe) {
      resetSize();
      setResizeMe(false);
    }
  }, [resetSize, resizeMe, setResizeMe]);
  useEffect(() => {
    let unlisten = null;
    listen('reset-size', () => {
      setResizeMe(true);
    }).then((func) => {
      unlisten = func;
    });
    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [setResizeMe]);

  return resetSize;
}

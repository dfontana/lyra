import { useCallback, useEffect, useState } from 'react'
import { appWindow, LogicalSize } from '@tauri-apps/api/window'

export default function useWindowResize(trigger) {
  const [initialRect, setInitialRect] = useState({ width: 0, height: 0 })
  const [hasInit, setHasInit] = useState(false)

  useEffect(() => {
    if (hasInit) {
      return
    }
    let rect = document.body.getBoundingClientRect()
    if (rect.width !== 0 && rect.height !== 0) {
      setInitialRect(rect)
      setHasInit(true)
    }
  }, [hasInit, setHasInit, setInitialRect])

  let resize = useCallback((rect) => {
    if (rect.width !== 0 && rect.height !== 0) {
      appWindow.setSize(new LogicalSize(rect.width, rect.height))
    }
  }, [])

  useEffect(() => {
    if (trigger !== null) {
      resize(document.body.getBoundingClientRect())
    }
  }, [trigger, resize])

  return useCallback(() => resize(initialRect), [initialRect, resize])
}

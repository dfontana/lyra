import React, { useState, useEffect } from "react";

export default function useKeyPress(targetKey, ref) {
  const [keyPressed, setKeyPressed] = useState(false);

  function downHandler(event) {
    if (event.key === targetKey) {
      setKeyPressed(true);
      event.preventDefault();
    }
  }

  const upHandler = (event) => {
    if (event.key === targetKey) {
      setKeyPressed(false);
      event.preventDefault();
    }
  };

  useEffect(() => {
    ref.current.addEventListener("keydown", downHandler);
    ref.current.addEventListener("keyup", upHandler);

    return () => {
      ref.current.removeEventListener("keydown", downHandler);
      ref.current.removeEventListener("keyup", upHandler);
    };
  }, [upHandler, downHandler]);

  return keyPressed;
}

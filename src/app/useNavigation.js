import { useEffect, useState } from 'react';
import { useKeyPressResetable } from './useKeyPress';

export default function useNavigation({ results, onSubmit }) {
  const [selection, setSelected] = useState(0);

  const [isArrowDown, resetDown] = useKeyPressResetable('ArrowDown');
  const [isArrowUp, resetUp] = useKeyPressResetable('ArrowUp');
  const [isEnter, resetEnter] = useKeyPressResetable('Enter');

  useEffect(() => {
    if (isArrowDown && selection < results.length - 1) {
      setSelected(selection + 1);
      resetDown();
    }
  }, [isArrowDown, resetDown, selection, setSelected, results]);

  useEffect(() => {
    if (isArrowUp && selection > 0) {
      setSelected(selection - 1);
      resetUp();
    }
  }, [isArrowUp, resetUp, selection, setSelected]);

  useEffect(() => {
    if (isEnter) {
      resetEnter();
      onSubmit(selection);
    }
  }, [isEnter, resetEnter, selection, onSubmit]);

  return [
    selection,
    () => {
      resetEnter();
      setSelected(0);
    },
  ];
}

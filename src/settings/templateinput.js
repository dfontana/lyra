import React, { useState } from 'react';
import { Input } from '@geist-ui/core';
import { invoke } from '@tauri-apps/api/tauri';
import debounce from './debounce';
const { VALIDATE_TEMPLATE } = window.__LYRA__.calls;

export default function TemplateInput({ setLock, setValue, initialValue, ...rest }) {
  const [valid, setValid] = useState(true);
  const ref = React.useRef(null);
  const validate_and_save = debounce(() => {
    invoke(VALIDATE_TEMPLATE, { input: ref.current.value })
      .then(() => {
        setValue(ref.current.value);
      })
      .then(() => {
        setValid(true);
        setLock(false);
      })
      .catch(() => {
        setValid(false);
        setLock(true);
      });
  }, 300);

  return (
    <Input
      ref={ref}
      type={valid ? '' : 'error'}
      initialValue={initialValue}
      scale={0.5}
      width="100%"
      onChange={validate_and_save}
      {...rest}
    />
  );
}

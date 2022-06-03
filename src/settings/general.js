import React, { useState, useCallback } from 'react';
import { Button, useToasts, Fieldset } from '@geist-ui/core';
import { invoke } from '@tauri-apps/api/tauri';
import TemplateInput from './templateinput';
import './bookmarklets.css';
const { SAVE_ENGINE } = window.__LYRA__.calls;

export default function General({ initialConfig }) {
  const { setToast } = useToasts();
  const [engine, setEngine] = useState(initialConfig.default_web_engine);
  const [lock, setLock] = useState(false);

  const saveForm = useCallback(() => {
    invoke(SAVE_ENGINE, { updates: engine })
      .then(() => setToast({ text: 'Saved!', type: 'success' }))
      .catch((err) => setToast({ text: `Error: ${err}`, type: 'error', delay: 10000 }));
  }, [engine, setToast]);

  return (
    <Fieldset>
      <Fieldset.Content>
        <TemplateInput
          setLock={setLock}
          setValue={setEngine}
          initialValue={engine}
          label="Default Web Query Template"
          placeholder="https://www.google.com/search?q={0}"
        />
      </Fieldset.Content>
      <Fieldset.Footer>
        <Button disabled={lock} type="success" auto scale={1 / 3} font="12px" onClick={saveForm}>
          Save
        </Button>
      </Fieldset.Footer>
    </Fieldset>
  );
}

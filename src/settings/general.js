import React, { useState, useCallback } from 'react';
import { Button, useToasts, Fieldset, Select, Divider, Grid, Input } from '@geist-ui/core';
import { invoke } from '@tauri-apps/api/tauri';
import TemplateInput from './templateinput';
import './bookmarklets.css';
import './general.css';
const { SAVE_ENGINE } = window.__LYRA__.calls;

const PLACEMENT_XY = 'XY';

export default function General({ initialConfig }) {
  const { setToast } = useToasts();
  const [engine, setEngine] = useState(initialConfig.default_web_engine);
  const [lock, setLock] = useState(false);

  const saveForm = useCallback(() => {
    invoke(SAVE_ENGINE, { updates: engine })
      .then(() => setToast({ text: 'Saved!', type: 'success' }))
      .catch((err) => setToast({ text: `Error: ${err}`, type: 'error', delay: 10000 }));
  }, [engine, setToast]);

  // TODO: implement form entries for the following
  // pub app_paths: Vec<PathBuf>,
  // pub app_extension: String,
  // pub result_count: usize
  // pub calc_trigger: String

  // TODO: initialConfig is not flowing correctly here, we need to get it reshaped to do so 
  //       (eg this is always defaulting)
  const [placement, setPlacement] = useState(initialConfig.placement || PLACEMENT_XY);
  const [placementXY, setPlacementXY] = useState(initialConfig.placement?.xy || { x: 0.0, y: 0.0 });

  const onPlacementChange = useCallback(
    (selected) => {
      setPlacement(selected);
    },
    [setPlacement]
  );

  const onChangeX = useCallback(
    (e) => {
      // Note: Will be empty string when not a valid number
      setPlacementXY((p) => ({ ...p, x: e.target.value }));
    },
    [setPlacementXY]
  );

  const onChangeY = useCallback(
    (e) => {
      setPlacementXY((p) => ({ ...p, y: e.target.value }));
    },
    [setPlacementXY]
  );

  const showXY = placement === PLACEMENT_XY ? false : 0;

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
        <Divider />
        <Grid.Container gap={2}>
          <Grid>
            <Select
              placeholder="Window Placement"
              initialValue={placement}
              onChange={onPlacementChange}
              disabled
            >
              <Select.Option label>Window Placement</Select.Option>
              <Select.Option value={PLACEMENT_XY}>Specific X, Y</Select.Option>
            </Select>
          </Grid>
          <Grid xs={showXY}>
            <Input
              htmlType="number"
              width="125px"
              label="X"
              placeholder="0.0"
              onChange={onChangeX}
              initialValue={placementXY.x}
            />
          </Grid>
          <Grid xs={showXY}>
            <Input
              htmlType="number"
              width="125px"
              label="Y"
              placeholder="0.0"
              onChange={onChangeY}
              initialValue={placementXY.y}
            />
          </Grid>
        </Grid.Container>
      </Fieldset.Content>
      <Fieldset.Footer>
        <Button disabled={lock} type="success" auto scale={1 / 3} font="12px" onClick={saveForm}>
          Save
        </Button>
      </Fieldset.Footer>
    </Fieldset>
  );
}

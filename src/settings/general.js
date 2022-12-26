import React, { useState, useCallback } from 'react';
import { Button, useToasts, Fieldset, Select, Grid, Input } from '@geist-ui/core';
// import { invoke } from '@tauri-apps/api/tauri';

import './general.css';
const PLACEMENT_XY = 'XY';

export default function General({ initialConfig }) {
  const { setToast } = useToasts();

  // TODO: UI - actually wire this into the initial config & write out, data needs reshaping
  //      (initialConfig is not flowing correctly here, we need to get it reshaped to do so)
  //      (eg this is always defaulting)

  const saveForm = useCallback(() => {
    setToast({text: 'Not impled', type: 'error', delay: 10000});
    // invoke(SAVE_..., { updates: ... })
    //   .then(() => setToast({ text: 'Saved!', type: 'success' }))
    //   .catch((err) => setToast({ text: `Error: ${err}`, type: 'error', delay: 10000 }));
  }, [setToast]);

  // TODO: UI - implement form entries for the following

  // pub app_paths: Vec<PathBuf>,
  // pub app_extension: String,
  // pub result_count: usize

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
        <Button type="success" auto scale={1 / 3} font="12px" onClick={saveForm}>
          Save
        </Button>
      </Fieldset.Footer>
    </Fieldset>
  );
}

import React, { useState, useRef, useCallback, useEffect } from 'react';
import {
  Button,
  Table,
  Tooltip,
  Avatar,
  Input,
  useClickAway,
  useToasts,
  Fieldset,
  useInput,
} from '@geist-ui/core';
import { X } from '@geist-ui/icons';
import { invoke } from '@tauri-apps/api/tauri';
const { IMAGE_TO_DATA, SAVE_BOOKMARKS } = window.__LYRA__.calls;

function newRow() {
  return {
    label: '',
    shortname: '',
    link: '',
    icon: '',
  };
}

function RenderIconBox(setData, _, rowData, rowIndex) {
  const ref = useRef();
  const [selected, setSelected] = useState(false);
  const { setToast } = useToasts();
  useClickAway(ref, () => setSelected(false));

  const onPaste = useCallback(
    (event) => {
      event.preventDefault();
      const url = (event.clipboardData || window.clipboardData).getData('text');
      invoke(IMAGE_TO_DATA, { url })
        .then((icon) => {
          setData((prev) =>
            prev.map((item, dataIndex) => (dataIndex !== rowIndex ? item : { ...item, icon }))
          );
        })
        .catch((err) => setToast({ text: err, type: 'error' }));
    },
    [setData, setToast, rowIndex]
  );

  const onClick = useCallback(() => setSelected((prev) => !prev), [setSelected]);

  return (
    <div
      ref={ref}
      style={{ outline: selected ? 'auto' : 'none' }}
      onClick={onClick}
      onPaste={onPaste}
    >
      <Avatar src={rowData.icon} style={{ background: 'black' }} />
    </div>
  );
}

function RenderRemoveButton(setData, rowIndex) {
  const removeHandler = useCallback(() => {
    // TODO this deletes the wrong items (last item in list it appears)
    setData((last) => last.filter((_, dataIndex) => dataIndex !== rowIndex));
  }, [setData, rowIndex]);
  return (
    <Button
      type="error"
      ghost
      iconRight={<X />}
      auto
      scale={1 / 3}
      px={0.6}
      onClick={removeHandler}
    />
  );
}

function RenderInput(setData, field, rowData, rowIndex) {
  const { state, bindings } = useInput(rowData[field]);
  useEffect(() => {
    setData((prev) => {
      prev[rowIndex][field] = state;
      return prev;
    });
  }, [state, setData, field, rowIndex]);
  return <Input {...bindings} />;
}

export default function BookmarkletManager({ initialConfig }) {
  console.log(initialConfig);
  const { setToast } = useToasts();
  const [data, setData] = useState(Object.values(initialConfig.bookmarks));

  const renderIconBox = useCallback(
    (v, row, idx) => RenderIconBox(setData, v, row, idx),
    [setData]
  );
  const renderDelete = useCallback((__, _, idx) => RenderRemoveButton(setData, idx), [setData]);
  const renderInput = useCallback(
    (field) => (_, row, idx) => RenderInput(setData, field, row, idx),
    [setData]
  );

  const addRow = useCallback(() => {
    setData((prev) => [...prev, newRow()]);
  }, [setData]);

  const saveForm = useCallback(() => {
    invoke(SAVE_BOOKMARKS, { bookmarks: data })
      .then(() => setToast({ text: 'Saved!', type: 'success' }))
      .catch((err) => setToast({ text: `Error: ${err}`, type: 'error', delay: 10000 }));
  }, [data, setToast]);

  return (
    <Fieldset>
      <Fieldset.Content>
        <Table data={data}>
          <Table.Column prop="label" width={175} render={renderInput('label')}>
            <Tooltip text="The label for this item in result list" placement="bottom">
              Label
            </Tooltip>
          </Table.Column>
          <Table.Column prop="shortname" width={75} render={renderInput('shortname')}>
            <Tooltip text="When searched will return this result" placement="bottom">
              Shortname
            </Tooltip>
          </Table.Column>
          <Table.Column prop="link" render={renderInput('link')}>
            <Tooltip
              text="Link that will open when selected. This will trigger your default browser."
              placement="bottom"
            >
              Opening Link
            </Tooltip>
          </Table.Column>
          <Table.Column prop="icon" width={50} render={renderIconBox}>
            <Tooltip
              text="An icon to display with the search result. Select the box and paste a url to a valid image."
              placement="bottom"
            >
              Icon
            </Tooltip>
          </Table.Column>
          <Table.Column width={25} render={renderDelete} />
        </Table>
      </Fieldset.Content>
      <Fieldset.Footer>
        <Button auto scale={1 / 3} font="12px" onClick={addRow}>
          Add
        </Button>
        <Button type="success" auto scale={1 / 3} font="12px" onClick={saveForm}>
          Save
        </Button>
      </Fieldset.Footer>
    </Fieldset>
  );
}

import React, { useState, useCallback } from 'react';
import { Button, Table, Tooltip, Avatar, Input, useToasts, Fieldset } from '@geist-ui/core';
import { X } from '@geist-ui/icons';
import { invoke } from '@tauri-apps/api/tauri';
import debounce from './debounce';
import './bookmarklets.css';
const { IMAGE_TO_DATA, SAVE_SEARCHERS, VALIDATE_TEMPLATE } = window.__LYRA__.calls;

function newRow() {
  return {
    label: '',
    shortname: '',
    template: '',
    icon: '',
  };
}

function RenderIconBox(setData, rowData, rowIndex) {
  const { setToast } = useToasts();

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

  const onKeyUp = useCallback(
    (e) => {
      if (e.key === 'Backspace') {
        setData((prev) =>
          prev.map((item, dataIndex) => (dataIndex !== rowIndex ? item : { ...item, icon: '' }))
        );
      }
    },
    [setData, rowIndex]
  );
  return (
    <div onPaste={onPaste} onKeyUp={onKeyUp} tabIndex="0" className="iconBox">
      <Avatar src={rowData.icon} style={{ background: 'black' }} />
    </div>
  );
}

function RenderRemoveButton(setData, rowIndex) {
  const removeHandler = useCallback(() => {
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

function TemplateInput(setLock, setData, rowData, rowIndex) {
  const [valid, setValid] = useState(true);
  const ref = React.useRef(null);
  const validate_and_save = debounce(() => {
    invoke(VALIDATE_TEMPLATE, { input: ref.current.value })
      .then(() => {
        setData((last) => {
          return last.map((item, dataIndex) => {
            if (dataIndex !== rowIndex) return item;
            return {
              ...item,
              template: ref.current.value,
            };
          });
        });
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
      initialValue={rowData.template}
      scale={0.5}
      width="100%"
      onChange={validate_and_save}
    />
  );
}

export default function SearcherManager({ initialConfig }) {
  const { setToast } = useToasts();
  const [data, setData] = useState(Object.values(initialConfig.searchers));
  const [lock, setLock] = useState(false);

  const renderIconBox = (_, row, idx) => RenderIconBox(setData, row, idx);
  const renderDelete = (__, _, idx) => RenderRemoveButton(setData, idx);
  const renderInput = (field) => (value, _, rowIndex) => {
    const onChange = (event) => {
      setData((last) => {
        return last.map((item, dataIndex) => {
          if (dataIndex !== rowIndex) return item;
          return {
            ...item,
            [field]: event.target.value,
          };
        });
      });
    };
    return <Input scale={0.5} width="100%" value={value} onChange={onChange} />;
  };
  const renderTemplateInput = (_, row, idx) => TemplateInput(setLock, setData, row, idx);

  const addRow = useCallback(() => {
    setData((prev) => [...prev, newRow()]);
  }, [setData]);

  const saveForm = useCallback(() => {
    invoke(SAVE_SEARCHERS, { updates: data })
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
          <Table.Column prop="link" render={renderTemplateInput}>
            <Tooltip
              text="Template link to hydrate during search. Templates are items like '{0}' whose number must be unique, between 0-9, and contiguous from 0. You may not repeat, but order can be any. This will trigger your default browser."
              placement="bottom"
            >
              Opening Template
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
        <Button disabled={lock} type="success" auto scale={1 / 3} font="12px" onClick={saveForm}>
          Save
        </Button>
      </Fieldset.Footer>
    </Fieldset>
  );
}

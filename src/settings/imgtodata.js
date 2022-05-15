import React, { useCallback, useEffect, useState } from 'react';
import { SubmitTextInput } from './textinput';
import { writeText } from '@tauri-apps/api/clipboard';
import { invoke } from '@tauri-apps/api/tauri';
import './textinput.scss';
const { IMAGE_TO_DATA } = window.__LYRA__.calls;

export default function ImgToData() {
  const [image, setImage] = useState('');
  const [alert, setAlert] = useState({ isOk: true, body: '' });
  const [imgSrc, setImgSrc] = useState('');

  const onChange = useCallback((event) => setImage(event.target.value), [setImage]);

  const onCopyClick = useCallback(() => {
    invoke(IMAGE_TO_DATA, { url: image })
      .then((data) => {
        writeText(data);
        setImgSrc(data);
      })
      .then(() => setAlert({ isOk: true, body: 'Copied!' }))
      .catch((err) => setAlert({ isOk: false, body: `Copy Failed: ${err}` }));
  }, [image, setImgSrc, setAlert]);

  useEffect(() => {
    if (image === '') {
      setAlert({ isOk: true, body: '' });
    }
  }, [image, setAlert]);

  return (
    <>
      <SubmitTextInput
        id="img-url"
        label="Image Url"
        buttonLabel="Copy"
        placeholder="Paste image url..."
        onChange={onChange}
        onClick={onCopyClick}
        value={image}
        isValid={image !== ''}
      >
        <div className="input-preview">{imgSrc !== '' && <img src={imgSrc} alt="" />}</div>
      </SubmitTextInput>
      {!alert.isOk && <div>{alert.body}</div>}
    </>
  );
}

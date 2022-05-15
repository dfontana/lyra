import React, { useCallback, useEffect, useState } from 'react';
import './settings.scss';
import { writeText } from '@tauri-apps/api/clipboard';
import { invoke } from '@tauri-apps/api/tauri';

const { IMAGE_TO_DATA } = window.__LYRA__.calls;

export default function Settings() {
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
    if (image == '') {
      setAlert({ isOk: true, body: '' });
    }
  }, [image, setAlert]);

  return (
    <div className="settingsRoot">
      <span className="input-container">
        <label for="img-url" className="input-label">
          Image Url
        </label>
        <input
          id="img-url"
          className="input-form"
          placeholder="Paste image url..."
          type="text"
          autoCorrect="off"
          onChange={onChange}
          value={image}
        />
        <button
          className={`input-submit ${alert.isOk ? '' : 'failed'}`}
          onClick={onCopyClick}
          disabled={image === ''}
        >
          Copy
        </button>
        <div className="input-preview">{imgSrc != '' && <img src={imgSrc} />}</div>
      </span>
      {!alert.isOk && <div>{alert.body}</div>}
    </div>
  );
}

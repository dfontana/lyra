import React, { useCallback, useState } from "react";
import { writeText } from '@tauri-apps/api/clipboard';

function getBase64FromImageUrl(url) {
  // TODO this doesn't actually really work due to cors
  //      Instead, what if we moved this functionality server-side:
  //      1) Fetch image with reqwest-like (GET url, accept: png-svg-jpeg-ico-svg, etc)
  //      2) Base64 encode the response
  //      3) build the datastring: data:image/{};base64,{} (where image could be png, svg+xml, etc)
  return new Promise((res, rej) => {
    var img = new Image();
    img.setAttribute('crossOrigin', 'anonymous');
    img.onload = function() {
      var canvas = document.createElement("canvas");
      canvas.width = this.width;
      canvas.height = this.height;
      var ctx = canvas.getContext("2d");
      ctx.drawImage(this, 0, 0);
      res(canvas.toDataURL("image/png"));
    };
    img.onerror = () => rej("Failed to load image");
    img.src = url;
  })
}

export default function Settings() {
  const [image, setImage] = useState("");

  const onChange = useCallback((event) => setImage(event.target.value), [setImage]);

  // TODO Clipboard for CMD+C/V is not working. Github issues says it has to 
  // have a menu bar, but that doesn't make sense since raycast don't ened it
  const onCopyClick = useCallback(() => {
    getBase64FromImageUrl(image)
      .then(writeText)
      .then(() => console.log("Copied!"))
      .catch(err => console.error(`Copy Failed: ${err}`))
  }, [image]);

  return (
    <div className="settingsRoot">
      <input
        className="imageConvert"
        placeholder="Paste image url..."
        type="text"
        autoCorrect="off"
        onChange={onChange}
        value={image}
      />
      <button onClick={onCopyClick}>Copy</button>
    </div>
  );
}

import React from 'react';
import ReactDOM from 'react-dom/client';
import './index.css';

function importPage() {
  switch (window.__LYRA_PAGE__) {
    case 'lyra-main':
      return import('./app/app');
    case 'lyra-settings':
      return import('./settings/settings');
    default:
      return Promise.reject('No such page: ' + window.__LYRA_PAGE__);
  }
}

importPage().then(({ default: Page }) =>
  ReactDOM.createRoot(document.getElementById('root')).render(
    <React.StrictMode>
      <Page />
    </React.StrictMode>
  )
);

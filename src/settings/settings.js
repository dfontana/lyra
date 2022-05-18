import React from 'react';
import ImgToData from './imgtodata';
import BookmarkletManager from './bookmarkletmanager';
import useHydration from './useHydration';
import { GeistProvider, CssBaseline, Page, Tabs, Loading } from '@geist-ui/core';

const { GET_CONFIG } = window.__LYRA__.calls;

export default function Settings() {
  const [isConfigLoading, config] = useHydration(GET_CONFIG);

  return (
    <GeistProvider>
      <CssBaseline />
      <Page dotBackdrop>
        {isConfigLoading ? (
          <Loading>Loading</Loading>
        ) : (
          <Tabs initialValue="1">
            <Tabs.Item label="bookmarks" value="1">
              <BookmarkletManager initialConfig={config} />
            </Tabs.Item>
            <Tabs.Item label="url" value="2">
              <ImgToData />
            </Tabs.Item>
          </Tabs>
        )}
      </Page>
    </GeistProvider>
  );
}

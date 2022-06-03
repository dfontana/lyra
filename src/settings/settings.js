import React from 'react';
import BookmarkletManager from './bookmarkletmanager';
import General from './general';
import SearcherManager from './searchermanager';
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
            <Tabs.Item label="general" value="1">
              <General initialConfig={config} />
            </Tabs.Item>

            <Tabs.Item label="bookmarks" value="2">
              <BookmarkletManager initialConfig={config} />
            </Tabs.Item>
            <Tabs.Item label="searchers" value="3">
              <SearcherManager initialConfig={config} />
            </Tabs.Item>
          </Tabs>
        )}
      </Page>
    </GeistProvider>
  );
}

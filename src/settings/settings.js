import React from 'react';
import ImgToData from './imgtodata';
import BookmarkletManager from './bookmarkletmanager';
import { GeistProvider, CssBaseline, Page, Tabs, Text } from '@geist-ui/core';

export default function Settings() {
  return (
    <GeistProvider>
      <CssBaseline />
      <Page dotBackdrop>
        <Tabs initialValue="1">
          <Tabs.Item label="bookmarks" value="1">
            <BookmarkletManager />
          </Tabs.Item>
          <Tabs.Item label="url" value="2">
            <ImgToData />
          </Tabs.Item>
        </Tabs>
      </Page>
    </GeistProvider>
  );
}

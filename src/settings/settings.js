import React from 'react'
import General from './general'
import SearcherManager from './searchermanager'
import useHydration from './useHydration'
import { GeistProvider, CssBaseline, Page, Tabs, Loading } from '@geist-ui/core'

const { GET_CONFIG } = window.__LYRA__.calls

export default function Settings() {
  const [isConfigLoading, config] = useHydration(GET_CONFIG)

  return (
    <GeistProvider>
      <CssBaseline />
      <Page dotBackdrop>
        {isConfigLoading ? (
          <Loading>Loading</Loading>
        ) : (
          <Tabs initialValue='1'>
            <Tabs.Item label='general' value='1'>
              <General initialConfig={config.general} />
            </Tabs.Item>
            {config?.webq && (
              <Tabs.Item label='webq' value='2'>
                <SearcherManager initialConfig={config.webq} />
              </Tabs.Item>
            )}
          </Tabs>
        )}
      </Page>
    </GeistProvider>
  )
}

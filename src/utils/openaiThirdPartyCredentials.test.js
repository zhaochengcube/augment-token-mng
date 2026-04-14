import test from 'node:test'
import assert from 'node:assert/strict'

import {
  buildOpenAIThirdPartyCredentialPreview,
  getAvailableOpenAIThirdPartyCredentialTemplates
} from './openaiThirdPartyCredentials/index.js'

const oauthAccount = {
  id: 'acc-1',
  email: 'user@example.com',
  account_type: 'oauth',
  chatgpt_account_id: 'fa8d225c-ee2a-4c1f-b4a8-16725740ddf6',
  token: {
    access_token: 'access-token-value',
    refresh_token: 'refresh-token-value',
    id_token: 'id-token-value'
  }
}

test('只返回当前账号可用的第三方凭证模板', () => {
  const templates = getAvailableOpenAIThirdPartyCredentialTemplates(oauthAccount)

  assert.equal(templates.length, 1)
  assert.equal(templates[0].id, 'cc-switch')
})

test('生成 cc-switch 凭证内容', () => {
  const preview = buildOpenAIThirdPartyCredentialPreview(oauthAccount, 'cc-switch', {
    now: '2026-04-11T15:22:16Z'
  })

  assert.deepEqual(JSON.parse(preview), {
    OPENAI_API_KEY: null,
    last_refresh: '2026-04-11T15:22:16Z',
    tokens: {
      access_token: 'access-token-value',
      account_id: 'fa8d225c-ee2a-4c1f-b4a8-16725740ddf6',
      id_token: 'id-token-value',
      refresh_token: 'refresh-token-value'
    }
  })
})

test('缺少必要 token 时不提供 cc-switch 模板', () => {
  const templates = getAvailableOpenAIThirdPartyCredentialTemplates({
    ...oauthAccount,
    token: {
      access_token: 'access-token-value',
      refresh_token: null,
      id_token: 'id-token-value'
    }
  })

  assert.deepEqual(templates, [])
})

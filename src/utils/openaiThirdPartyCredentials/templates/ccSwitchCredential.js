const formatIsoSeconds = (value = new Date()) => {
  const date = value instanceof Date ? value : new Date(value)
  return date.toISOString().replace(/\.\d{3}Z$/, 'Z')
}

const parseOpenAIAuthJson = (account) => {
  if (!account?.openai_auth_json) return null
  try {
    return JSON.parse(account.openai_auth_json)
  } catch {
    return null
  }
}

const resolveAccountId = (account) => {
  const authInfo = parseOpenAIAuthJson(account)
  return (
    account?.chatgpt_account_id ??
    authInfo?.chatgpt_account_id ??
    authInfo?.['https://api.openai.com/auth']?.chatgpt_account_id ??
    null
  )
}

const hasRequiredTokens = (account) => {
  const token = account?.token
  return Boolean(
    account?.account_type !== 'api' &&
      token?.access_token &&
      token?.refresh_token &&
      token?.id_token
  )
}

export default {
  id: 'cc-switch',
  label: 'cc-switch',
  supports(account) {
    return hasRequiredTokens(account)
  },
  build(account, context = {}) {
    if (!hasRequiredTokens(account)) {
      throw new Error('Account does not support cc-switch credentials')
    }

    return JSON.stringify(
      {
        OPENAI_API_KEY: null,
        last_refresh: formatIsoSeconds(context.now),
        tokens: {
          access_token: account.token.access_token,
          account_id: resolveAccountId(account),
          id_token: account.token.id_token,
          refresh_token: account.token.refresh_token
        }
      },
      null,
      2
    )
  }
}

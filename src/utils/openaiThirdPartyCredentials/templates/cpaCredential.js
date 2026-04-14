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
      token?.refresh_token
  )
}

export default {
  id: 'cpa',
  label: 'CPA',
  supports(account) {
    return hasRequiredTokens(account)
  },
  build(account) {
    if (!hasRequiredTokens(account)) {
      throw new Error('Account does not support CPA credentials')
    }

    return JSON.stringify(
      {
        type: 'codex',
        email: account.email || '',
        account_id: resolveAccountId(account) || '',
        access_token: account.token.access_token,
        refresh_token: account.token.refresh_token,
        id_token: account.token.id_token || ''
      },
      null,
      2
    )
  }
}

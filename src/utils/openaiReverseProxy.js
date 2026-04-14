const isOAuthAccount = (account) => account?.account_type !== 'api'

export const getReverseProxyAction = (account) => {
  if (!isOAuthAccount(account)) {
    return null
  }

  return account.reverse_proxy_enabled !== false ? 'disable' : 'enable'
}

export const toggleReverseProxyForAccount = (
  account,
  timestamp = Math.floor(Date.now() / 1000)
) => {
  if (!isOAuthAccount(account)) {
    return false
  }

  account.reverse_proxy_enabled = !(account.reverse_proxy_enabled !== false)
  account.updated_at = timestamp
  return true
}

export const applyReverseProxyToSelection = (
  accounts,
  selectedAccountIds,
  enabled,
  timestamp = Math.floor(Date.now() / 1000)
) => {
  let updatedCount = 0

  for (const account of accounts) {
    if (!selectedAccountIds.has(account.id) || !isOAuthAccount(account)) {
      continue
    }

    account.reverse_proxy_enabled = enabled
    account.updated_at = timestamp
    updatedCount++
  }

  return updatedCount
}

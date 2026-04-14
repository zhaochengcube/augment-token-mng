import ccSwitchCredential from './templates/ccSwitchCredential.js'

const templates = [ccSwitchCredential]

export const getAvailableOpenAIThirdPartyCredentialTemplates = (account) => {
  return templates.filter((template) => template.supports(account))
}

export const getOpenAIThirdPartyCredentialTemplateById = (templateId) => {
  return templates.find((template) => template.id === templateId) || null
}

export const buildOpenAIThirdPartyCredentialPreview = (account, templateId, context = {}) => {
  const template = getOpenAIThirdPartyCredentialTemplateById(templateId)
  if (!template) {
    throw new Error(`Unknown third-party credential template: ${templateId}`)
  }
  return template.build(account, context)
}

# Email

Manage Outlook, GPT mail and iCloud HME email features.

## Features

### iCloud HME

Generate, list, deactivate and clean up iCloud Hide My Email addresses (requires iCloud+).

### Outlook

Outlook mailbox account management:

- **Add accounts**: Manual import (email----Refresh Token----Client ID) or OAuth authorization; supports batch import.
- **Azure app registration (OAuth)**: Add `http://localhost:8080` under **Mobile and desktop applications**.
- **Token management**: Background automatic token refresh with manual bulk refresh and failed-retry support.
- **Status checks**: Batch check account status (active / error / banned).
- **Email viewing**: Click an account to view inbox messages (via Microsoft Graph API).

### GPTMail

Generate temporary email addresses and receive mail via the GPTMail API:

- **Generate email**: One-click random email address generation with custom API key support.
- **Receive mail**: Enter an email address to query messages; verification codes are extracted automatically.
- **Auto-polling**: Enable automatic periodic checking for new emails.
- **History**: Generated addresses are saved automatically with search and management.

Open from the sidebar: **Email**, then choose the service.

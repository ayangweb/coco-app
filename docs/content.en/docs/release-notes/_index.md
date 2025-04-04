---
weight: 80
title: "Release Notes"
---

# Release Notes

Information about release notes of Coco Server is provided here.

## Latest (In development)

### Breaking changes

### Features

- Linux support for application search #330

### Bug fix

### Improvements

## 0.3.0 (2025-03-31)

### Breaking changes

- feat: add web pages components #277
- feat: support for customizing some of the preset shortcuts #316

### Features

- feat: support multi websocket connections #314
- feat: add support for embeddable web widget #277

### Bug fix

### Improvements

- refactor: refactor invoke related code #309
- refactor: hide apps without icon #312

## 0.2.1 (2025-03-14)

### Features

- support for automatic in-app updates #274

### Breaking changes

### Bug fix

- Fix the issue that the fusion search include disabled servers
- Fix incorrect version type: should be string instead of u32
- Fix the chat end judgment type #280
- Fix the chat scrolling and chat rendering #282
- Fix: store data is not shared among multiple windows #298

### Improvements

- Refactor: chat components #273
- Feat：add endpoint display #282
- Chore: chat window min width & remove input bg #284
- Chore: remove selected function & add hide_coco #286
- Chore：websocket timeout increased to 2 minutes #289
- Chore: remove chat input border & clear input #295

## 0.2.0 (2025-03-07)

### Features

- Add timeout to fusion search #174
- Add api to disable or enable server #185
- Networked search supports selection of data sources #209
- Add deepthink and knowledge search options to RAG based chat
- Support i18n, add Chinese language support
- Support Windows platform
- etc.

### Breaking changes

### Bug fix

- Fix to access deeplink for linux #148
- etc.

### Improvements

- Improve app startup, init application search in background #172
- Refactoring login #173
- Init icons in background during start #176
- Refactoring health api #187
- Refactoring assistant api #195
- Refactor: remove websocket_session_id from message request #206
- Refactor: the display of search results and the logic of creating new chats #207
- Refactor: AI conversation rendering logic #216
- Refresh all server's info on purpose, get the actual health info #225
- Improve chat message display
- Improve application search, support macOS/Windows and Linux
- Display the version of the server in the settings page
- Allow to switch between different data sources in networked search
- Allow to switch servers in the settings page
- etc.

## 0.1.0 (2025-02-16)

### Features

- Fusion Search
- Chat with AI Assistant
- RAG-based AI Chat
- General Settings
- Global Shortcut
- Auto Start on Startup
- Shortcut to Features
- Application Search for macOS
- Option to Connect to Self-Hosted Coco Server

### Breaking changes

### Bug fix

### Improvements

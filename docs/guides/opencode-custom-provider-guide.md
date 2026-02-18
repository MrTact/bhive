# OpenCode Custom Service Provider (Relay Station) Integration Guide

**Author:** webarn
**Posted:** December 17, 2025
**Views:** 7.3k | **Likes:** 230

---

## Introduction

Recently, I've noticed many friends are interested in OpenCode but hesitate because they don't know how to integrate custom API relay stations. After some experimentation, I successfully integrated a relay station with OpenCode. Here's the complete process, hoping it helps those who need it!

---

## Prerequisites

- OpenCode CLI installed (can be installed via `npm install -g @opencode/cli`). The official website also provides other installation methods.

---

## Integration Steps

### 1. Initialize Custom Service Provider

**Do NOT start OpenCode directly.** Instead, run the following command in the terminal:

```bash
opencode auth login
```

- In the provider list, select **`other`** (it's at the bottom; you can search for it directly).
- The system will prompt you to enter a **Provider ID**: Enter a unique identifier (e.g., `myproxy`). This must be used consistently in subsequent configurations.
- Then enter the **API Key**: You can enter any content (e.g., `dummy`) because the actual key can be securely referenced through the configuration file (see next step).

**Purpose of this step:** This registers a custom service provider in OpenCode's local credential manager, making it easy to reference the key later.

---

### 2. Configure Relay Station API Address

Open the OpenCode configuration directory (path varies by system):

- **macOS / Linux**: `~/.config/opencode/`
- **Windows**: `Users\***\.config\opencode` (provided by community members)

In that directory, create or edit the configuration file: **`opencode.json`** with the following content:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "provider": {
    "myproxy": {
      // Must match the Provider ID from step 1 exactly!
      "npm": "@ai-sdk/openai-compatible",
      "name": "My Relay Station", // Display name in UI, customizable
      "options": {
        "baseURL": "https://your-proxy-domain.com/v1" // Your relay station API address (must end with /v1 or follow OpenAI format)
        // "apiKey": "{cred:myproxy}"  // Optional: automatically references the key stored in step 1 (recommended, avoids plaintext keys)
        // If the relay station requires custom headers, you can add:
        // "headers": {
        //   "X-Custom-Header": "your-value"
        // }
      },
      "models": {
        "gpt-4o": {
          // Model IDs supported by the relay station, e.g., gpt-4o, claude-3-5-sonnet, etc.
          "name": "GPT-4o (Relay)"
        },
        "claude-3-5-sonnet-20241022": {
          "name": "Claude 3.5 Sonnet"
        }
        // Add more models...
      }
    }
  }
}
```

**Key Points:**

- **`myproxy`** must **exactly match** the Provider ID entered in step 1.
- **`baseURL`** must point to the relay station's OpenAI-compatible API endpoint, usually ending with `/v1` (if it doesn't work, try removing `/v1` for testing).
- **`apiKey: "{cred:myproxy}"`** will automatically read the key saved during `opencode auth login`, **no need to write it plaintext in the config file** – more secure!

---

### 3. Restart and Verify

After saving the configuration, start OpenCode:

```bash
opencode
```

In the chat interface, enter the command:

```
/models
```

You should see your configured relay station and its models (e.g., "GPT-4o (Relay)") in the model list. Select it and you can use it normally!

---

## Important Notes

1. **Provider ID must be consistent**: The key name in the config file (e.g., `myproxy`) must exactly match the ID entered during `auth login` (case-sensitive).

2. **API address format**: Ensure `baseURL` is correct. You can first test if the relay station responds to `/v1/models` using curl or Postman.

3. **Model ID must match**: The keys under `models` (e.g., `gpt-4o`) must match the model IDs actually supported by the relay station.

4. If changes don't take effect after modifying the config, try completely exiting OpenCode and restarting.

**Hope this helps! Please give some encouragement if the configuration was successful!**

---

## Summary

This is a comprehensive tutorial for integrating custom API relay stations (like proxy services for OpenAI or Claude APIs) into OpenCode, which appears to be a CLI tool similar to Claude Code. The guide provides step-by-step instructions for authentication, configuration, and verification.

---

**Source:** https://linux.do/t/topic/1329050

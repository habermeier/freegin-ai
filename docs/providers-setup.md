# AI Provider Setup Guide

This guide explains how to set up API keys for the various AI providers supported by freegin-ai.

## Interactive Setup (Easiest)

The fastest way to get started is with the interactive setup wizard:

```bash
freegin-ai --init
```

This will:
- Walk you through each provider with sign-up URLs
- Let you skip providers you don't want (just press Enter)
- Skip providers you've already configured
- Store all keys securely with encryption
- Can be re-run anytime to add more providers

## Quick Start - Recommended Providers

### 🚀 Groq (Recommended - Fastest)
- **Free Tier**: 14,400 requests/day, 6,000 tokens/minute
- **Speed**: 0.13s first token latency (fastest available)
- **Best For**: Quick responses, high-volume testing
- **Get API Key**: https://console.groq.com/keys
- **Setup (Encrypted Storage - Recommended)**:
  ```bash
  freegin-ai add-service groq
  # Follow prompts to enter API key securely
  ```

### 🎯 DeepSeek (Low Cost Pay-As-You-Go)
- **Pricing**: Pay-per-use ($0.028/M input tokens, $2.19/M output for R1)
- **Models**: DeepSeek-V3, DeepSeek-R1 (rivals OpenAI o1)
- **Best For**: Affordable reasoning tasks, low-cost inference
- **Get API Key**: https://platform.deepseek.com/api_keys
- **Note**: API requires payment (very cheap), web chat interface is free
- **Setup (Encrypted Storage - Recommended)**:
  ```bash
  freegin-ai add-service deepseek
  # Follow prompts to enter API key securely
  ```

### 🤝 Together AI
- **Pricing**: Requires $5 deposit to access API
- **Free Models**: Llama 3.3 70B and other free-tier models available after deposit
- **Best For**: Diverse model access
- **Get API Key**: https://api.together.xyz/settings/api-keys
- **Setup (Encrypted Storage - Recommended)**:
  ```bash
  freegin-ai add-service together
  # Follow prompts to enter API key securely
  # Note: Requires $5 deposit to activate API access
  ```

## Additional Providers

### Google Gemini
- **Free Tier**: 60 requests/min, 1M tokens/min
- **Models**: Gemini 2.0 Flash
- **Get API Key**: https://makersuite.google.com/app/apikey
- **Setup (Encrypted Storage - Recommended)**:
  ```bash
  freegin-ai add-service google
  ```

### Hugging Face
- **Free Tier**: Rate-limited serverless API
- **Get API Key**: https://huggingface.co/settings/tokens
- **Setup (Encrypted Storage - Recommended)**:
  ```bash
  freegin-ai add-service huggingface
  ```

### OpenAI
- **Pricing**: Pay-as-you-go (no free tier)
- **Get API Key**: https://platform.openai.com/api-keys
- **Setup (Encrypted Storage - Recommended)**:
  ```bash
  freegin-ai add-service openai
  ```

### Anthropic Claude
- **Pricing**: Pay-as-you-go (limited free credits)
- **Get API Key**: https://console.anthropic.com/
- **Setup (Encrypted Storage - Recommended)**:
  ```bash
  freegin-ai add-service anthropic
  ```

### Cohere
- **Free Tier**: Available for experimentation
- **Get API Key**: https://dashboard.cohere.com/api-keys
- **Setup (Encrypted Storage - Recommended)**:
  ```bash
  freegin-ai add-service cohere
  ```

## Alternative: Config File Method

You can also add API keys directly to `~/.config/freegin-ai/config.toml` (less secure):

```toml
[providers.groq]
api_key = "YOUR_KEY_HERE"
api_base_url = "https://api.groq.com/openai/v1"
```

**Note**: Encrypted storage using `add-service` is more secure and recommended for all providers.

## Default Models by Provider

The system automatically seeds these models for each provider:

### Groq
- **Chat**: `llama-3.3-70b-versatile` (priority 10)
- **Code**: `llama-3.3-70b-versatile` (priority 10)
- **Summarization**: `llama-3.3-70b-versatile` (priority 20)
- **Creative**: `llama-3.3-70b-versatile` (priority 15)

### DeepSeek
- **All Workloads**: `deepseek-chat` (priorities 15-25)
- Supports: Chat, Code, Summarization, Extraction, Creative, Classification

### Together AI
- **Chat**: `meta-llama/Llama-3.3-70B-Instruct-Turbo-Free` (priority 30)
- **Code**: `meta-llama/Llama-3.3-70B-Instruct-Turbo-Free` (priority 25)

### Google Gemini
- **Chat**: `gemini-2.0-flash` (priority 40)
- **Code**: `gemini-2.0-flash` (priority 35)
- **Summarization**: `gemini-2.0-flash` (priority 40)

## Priority System

Lower priority numbers are tried first:
- **10-20**: Groq (fastest, truly free)
- **15-25**: DeepSeek (pay-per-use, very cheap)
- **30**: Together AI (requires $5 deposit)
- **35-40**: Google Gemini (rate-limited free)

## Testing Your Setup

```bash
# Check which providers are configured
freegin-ai list-services

# Check provider health and models
freegin-ai status

# Test a provider
freegin-ai generate --prompt "Hello, world!" --provider groq

# Let the router choose automatically
freegin-ai generate --prompt "Hello, world!"
```

## Troubleshooting

### Provider shows "none" in list-services
- Check that `api_key` field is not empty in config.toml
- Ensure config file is at `~/.config/freegin-ai/config.toml`

### Provider is degraded/unavailable
- Run `freegin-ai status` to see error details
- Wait for retry window to expire
- Check API key is valid
- Verify API endpoint URL is correct

### All providers unavailable
- At least one provider must be configured with a valid API key
- Check `freegin-ai list-services` shows at least one provider as "environment"
- Review `freegin-ai status` for health information

## Multi-Provider Strategy

For best reliability and cost optimization:

1. **Primary**: Groq (fast, generous free tier)
2. **Backup**: DeepSeek (very low cost, great for complex reasoning)
3. **Fallback**: Together AI or Google Gemini
4. **Production**: Consider paid tiers for mission-critical workloads

The health tracking system automatically handles failover between providers!
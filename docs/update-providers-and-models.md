# Provider and Model Update Guide

This document provides a comprehensive workflow for researching, evaluating, and updating AI providers and models in the freegin-ai project. Run this process periodically (monthly/quarterly) to keep provider information current.

---

## 🎯 Objective

Find the best **free or low-cost** AI providers with generous limits and update the project to use current model names, accurate pricing information, and optimal routing priorities.

---

## 📋 Research Phase

### Step 1: Web Research Strategy

Use web search to gather current information about AI providers. Focus on **recent** information (last 3-6 months).

#### Primary Search Queries

Execute these searches and compile results:

```
1. "free AI API 2025" OR "free LLM API 2025"
2. "AI provider comparison free tier 2025"
3. "Groq API free tier limits 2025"
4. "DeepSeek API pricing 2025"
5. "Together AI free models 2025"
6. "Google Gemini API free tier 2025"
7. "Hugging Face Inference API free 2025"
8. "best free AI APIs reddit 2025"
9. "LLM API rate limits comparison"
10. "new AI providers 2025"
```

#### Information Sources (Priority Order)

1. **Official documentation** (highest priority)
   - Provider official docs (api.groq.com/docs, platform.deepseek.com/docs, etc.)
   - Pricing pages on official websites
   - Official announcement blogs

2. **Community resources**
   - Reddit: r/LocalLLaMA, r/MachineLearning, r/OpenAI
   - Hacker News discussions
   - Dev.to and Medium articles (last 6 months only)

3. **Comparison sites**
   - artificialanalysis.ai
   - LLM provider comparison sites
   - GitHub awesome-lists for LLMs

4. **Developer forums**
   - GitHub issues on provider SDK repositories
   - Stack Overflow recent questions
   - Discord/Slack communities (if accessible)

### Step 2: Provider Evaluation Criteria

For each provider found, gather this information:

#### Essential Information
- [ ] **Provider Name**: Official name and common aliases
- [ ] **API Base URL**: Current endpoint (e.g., https://api.groq.com/openai/v1)
- [ ] **Authentication**: API key location and format
- [ ] **Free Tier Details**:
  - Requests per day/minute/month
  - Tokens per request/minute
  - Rate limits
  - Any deposit requirements ($5, etc.)
  - Expiration of free tier (if any)
- [ ] **Model Names**: Current model identifiers (CRITICAL - these change!)
- [ ] **API Compatibility**: OpenAI-compatible? Custom format?
- [ ] **Reliability**: Service uptime, known issues
- [ ] **Speed**: Typical latency (tokens/second)

#### Quality Indicators
- [ ] **Recent Activity**: Provider still active? (check last 30 days)
- [ ] **Community Feedback**: What are users saying?
- [ ] **Model Quality**: Benchmark scores (if available)
- [ ] **Stability**: Frequent API changes? Breaking changes?

#### Red Flags (Skip These)
- ❌ Requires credit card for "free" tier without clear limits
- ❌ Undocumented API changes frequently
- ❌ Poor community reputation
- ❌ Service frequently down
- ❌ Free tier severely limited (<100 requests/day)

### Step 3: Model Name Verification

**CRITICAL**: Model names change frequently. Verify current names.

#### For Each Provider:

1. **Check Official Model List**
   ```
   Search: "[provider name] API model list 2025"
   Search: "[provider name] available models documentation"
   ```

2. **Verify via API (if you have access)**
   ```bash
   # Groq example
   curl https://api.groq.com/openai/v1/models \
     -H "Authorization: Bearer $GROQ_API_KEY"

   # DeepSeek example
   curl https://api.deepseek.com/models \
     -H "Authorization: Bearer $DEEPSEEK_API_KEY"
   ```

3. **Check Recent GitHub Issues**
   ```
   Search: "[provider name] model not found" site:github.com
   Search: "[provider name] deprecated models" site:github.com
   ```

4. **Cross-reference with Community**
   ```
   Search: "[model name] [provider name] working" site:reddit.com
   ```

#### Common Model Naming Patterns

- **Groq**: `llama-3.3-70b-versatile`, `llama-3.1-70b-versatile`, `mixtral-8x7b-32768`
- **DeepSeek**: `deepseek-chat`, `deepseek-coder`, `deepseek-reasoner`
- **Together AI**: `meta-llama/Llama-3.3-70B-Instruct-Turbo-Free`, `mistralai/Mixtral-8x7B-Instruct-v0.1`
- **Google Gemini**: `gemini-2.0-flash-exp`, `gemini-1.5-pro`, `gemini-1.5-flash`
- **OpenAI**: `gpt-4o`, `gpt-4o-mini`, `gpt-3.5-turbo`

**Note**: Free tier models often have "free" or "turbo" in the name (Together AI pattern).

---

## 🔍 Information Compilation Template

Create a research document with this structure:

```markdown
# Provider Research - [Date]

## Current Providers (Review)

### Groq
- **Status**: [Active/Deprecated/Changed]
- **Free Tier**: [Current limits]
- **Models**: [List current model names]
- **Changes**: [What changed since last update]
- **Source**: [URLs for verification]

### DeepSeek
- **Status**: [Active/Deprecated/Changed]
- **Free Tier**: [Current limits]
- **Models**: [List current model names]
- **Changes**: [What changed since last update]
- **Source**: [URLs for verification]

[Repeat for all current providers]

## New Providers Found

### [Provider Name]
- **Why Add**: [Reasoning - better free tier? unique models?]
- **Free Tier**: [Details]
- **Models**: [Available models]
- **API Format**: [OpenAI-compatible? Custom?]
- **Priority**: [Suggested priority 1-100]
- **Source**: [URLs for verification]

## Deprecated/Removed Providers

### [Provider Name]
- **Reason**: [Why removing - no longer free? service shut down?]
- **Migration Path**: [Alternative provider suggestion]

## Pricing Changes

### [Provider Name]
- **Old**: [Previous pricing/limits]
- **New**: [Current pricing/limits]
- **Impact**: [How this affects our users]
```

---

## 💻 Code Update Phase

Once research is complete, update the codebase in this order:

### 1. Update Provider Enum (if new providers)

**File**: `src/providers/mod.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Provider {
    OpenAI,
    Google,
    HuggingFace,
    Anthropic,
    Cohere,
    Groq,
    DeepSeek,
    Together,
    NewProvider,  // Add here
}
```

Update `as_str()` and `from_alias()` methods:

```rust
pub fn as_str(&self) -> &'static str {
    match self {
        // ... existing
        Provider::NewProvider => "newprovider",
    }
}

pub fn from_alias(alias: &str) -> Option<Self> {
    match alias.to_lowercase().as_str() {
        // ... existing
        "newprovider" | "new-provider" => Some(Provider::NewProvider),
        _ => None,
    }
}
```

### 2. Create Provider Client (if new provider)

**File**: `src/providers/newprovider.rs`

Use existing providers as templates:
- OpenAI-compatible → Copy from `groq.rs` or `deepseek.rs`
- Custom API → Copy from `google.rs` and adapt

Basic structure:

```rust
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    error::AppError,
    models::{AIRequest, AIResponse},
    providers::{AIProvider, Provider},
};

pub struct NewProviderClient {
    api_key: String,
    base_url: String,
    http_client: Client,
}

impl NewProviderClient {
    pub fn new(api_key: String, base_url: String) -> Result<Self, AppError> {
        if api_key.trim().is_empty() {
            return Err(AppError::ConfigError("API key cannot be empty".into()));
        }
        Ok(Self {
            api_key,
            base_url,
            http_client: Client::new(),
        })
    }
}

#[async_trait]
impl AIProvider for NewProviderClient {
    async fn generate(&self, request: &AIRequest) -> Result<AIResponse, AppError> {
        // Implement API call
    }
}
```

**Don't forget**: Add `pub mod newprovider;` to `src/providers/mod.rs`

### 3. Update Configuration

**File**: `src/config.rs`

Add provider config struct:

```rust
pub struct ProvidersConfig {
    // ... existing
    pub newprovider: Option<ProviderDetails>,
}
```

**File**: `.config/template.toml`

Add configuration section:

```toml
[providers.newprovider]
# Sign up: https://provider.com/api-keys
# Free tier: [describe limits]
api_key = ""
api_base_url = "https://api.provider.com/v1"
```

### 4. Update Router Initialization

**File**: `src/providers/router.rs`

Add provider initialization in `from_config()`:

```rust
// NewProvider - check encrypted storage first, then config
let newprovider_cfg = config.providers.newprovider.as_ref();
let newprovider_token_cfg = newprovider_cfg.and_then(|cfg| {
    let trimmed = cfg.api_key.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
});
let newprovider_token = match newprovider_token_cfg {
    Some(token) => Some(token),
    None => store.get_token(Provider::NewProvider).await?,
};
if let Some(token) = newprovider_token {
    let base_url = store
        .resolve_base_url(
            Provider::NewProvider,
            newprovider_cfg.map(|cfg| cfg.api_base_url.as_str()),
        )
        .to_string();
    let client = NewProviderClient::new(token, base_url)?;
    drop(providers.insert(Provider::NewProvider, Arc::new(client)));
    fallback_order.push(Provider::NewProvider);
} else {
    debug!(provider = "newprovider", "Provider not configured (missing credentials)");
}
```

Add to imports at top of file:
```rust
use super::{
    // ... existing
    newprovider::NewProviderClient,
};
```

### 5. Update Model Catalog

**File**: `src/catalog.rs`

Update `seed_defaults()` function with new models:

```rust
let defaults = vec![
    // Groq - UPDATE MODEL NAMES HERE
    (
        Provider::Groq,
        Workload::Chat,
        "llama-3.3-70b-versatile",  // ← Verify this is current!
        10,
        "Fast, versatile Llama model",
    ),

    // New provider - ADD NEW MODELS HERE
    (
        Provider::NewProvider,
        Workload::Chat,
        "provider-model-name",
        15,  // Priority: lower = tried first
        "Description of model",
    ),
];
```

**Priority Guidelines**:
- 1-10: Ultra-fast, truly free providers (Groq)
- 11-20: Unlimited free or very generous (DeepSeek)
- 21-30: Free with deposit or good limits (Together AI)
- 31-50: Rate-limited free tiers (Google Gemini, HuggingFace)
- 51+: Paid tiers, fallback options

### 6. Update CLI Commands

**File**: `src/main.rs`

#### Add to `handle_add_service()`:

```rust
async fn handle_add_service(provider: Provider, store: &CredentialStore) -> Result<(), AppError> {
    let (url, prompt) = match provider {
        // ... existing providers
        Provider::NewProvider => (
            "https://provider.com/api-keys",
            "Enter NewProvider API key (input hidden): ",
        ),
    };
    // ... rest of function
}
```

#### Add to `handle_init()`:

```rust
let providers = vec![
    // ... existing providers
    (
        Provider::NewProvider,
        "NewProvider",
        "Description of free tier and features",
        "https://provider.com/api-keys",
    ),
];
```

#### Update help text in `print_help()`:

```rust
Providers:
  groq             Groq (ultra-fast, 14.4K requests/day free) - https://console.groq.com/keys
  // ... existing
  newprovider      NewProvider (description) - https://provider.com/api-keys
```

### 7. Update Health Tracking

**File**: `src/health.rs`

Add to `get_all_health()`:

```rust
pub async fn get_all_health(&self) -> Result<Vec<ProviderHealth>, AppError> {
    let providers = [
        // ... existing
        Provider::NewProvider,
    ];
    // ... rest
}
```

### 8. Update Documentation

#### **File**: `docs/providers-setup.md`

Add detailed provider setup section:

```markdown
### 🆕 NewProvider
- **Free Tier**: [Describe limits]
- **Models**: [List key models]
- **Best For**: [Use cases]
- **Get API Key**: https://provider.com/api-keys
- **Setup (Encrypted Storage - Recommended)**:
  ```bash
  freegin-ai add-service newprovider
  # Follow prompts to enter API key securely
  ```
```

Update provider comparison table and priority system section.

#### **File**: `README.md`

Update provider comparison table:

```markdown
| Provider | Free Tier | Speed | Best For |
|----------|-----------|-------|----------|
| ... existing ...
| **NewProvider** | [limits] | [speed] | [use case] |
```

Update "Free Tier Focus" section if relevant.

---

## 🧪 Testing Phase

### 1. Build and Install

```bash
# Clean build
cargo clean
cargo build --release

# Check for warnings
cargo clippy -- -D warnings

# Install locally
make install
```

### 2. Verify Configuration

```bash
# Check help text includes new provider
freegin-ai --help | grep -i newprovider

# Run interactive setup
freegin-ai --init
# Verify new provider appears with correct URL

# Check template
cat ~/.config/freegin-ai/config.toml | grep -A 3 newprovider
```

### 3. Test Provider Addition

```bash
# Add the provider (requires valid API key)
freegin-ai add-service newprovider

# Verify it's stored
freegin-ai list-services | grep newprovider
# Should show: newprovider: stored
```

### 4. Test Model Catalog

```bash
# Check default models were seeded
freegin-ai list-models --provider newprovider

# Should show models for different workloads
```

### 5. Test Generation

```bash
# Test with explicit provider
freegin-ai generate --provider newprovider --prompt "Say hello" --verbose

# Should output:
# === Metadata ===
# Provider: newprovider
#
# === Response ===
# Hello! [or provider response]

# Test automatic routing (if model name is unique)
freegin-ai generate --model "provider-model-name" --prompt "Say hello"
```

### 6. Test Health Tracking

```bash
# Check provider health
freegin-ai status --provider newprovider

# Should show:
# ═══ NEWPROVIDER ✓ AVAILABLE ═══
# [model info and usage stats]
```

### 7. Test Error Handling

```bash
# Test with invalid API key (expect graceful failure)
freegin-ai remove-service newprovider
# Edit config to add invalid key
freegin-ai generate --provider newprovider --prompt "test"
# Should show clear error, mark provider as degraded

# Check health shows the error
freegin-ai status --provider newprovider
# Should show: ⚠ DEGRADED with error message
```

### 8. Test Model Name Updates

For existing providers, verify model names still work:

```bash
# Test each provider with default model
for provider in groq deepseek together google; do
  echo "Testing $provider..."
  freegin-ai generate --provider $provider --prompt "Say OK" --verbose
done

# Any 404 "model not found" errors indicate model names need updating
```

---

## 📝 Documentation Phase

### Update CHANGELOG (if exists)

```markdown
## [Unreleased]

### Added
- NewProvider support with free tier (X requests/day)
- Models: provider-model-name (chat, code)

### Changed
- Updated Groq model names from X to Y
- Updated DeepSeek free tier limits (now unlimited)
- Provider priority: DeepSeek now priority 15 (from 20)

### Deprecated
- OldProvider (service discontinued as of [date])

### Fixed
- Model names for Groq updated to current API
```

### Update GitHub Issues/Discussions (if applicable)

Create issue documenting changes:

```markdown
Title: Provider Update [Date] - Model Names and New Providers

## Summary
Updated provider information based on current documentation and testing.

## Changes Made
- ✅ Verified Groq model names (still current)
- ✅ Updated DeepSeek limits (now unlimited)
- ✅ Added NewProvider with free tier
- ⚠️ Removed OldProvider (service shut down)

## Testing
All providers tested and verified working as of [date].

## Migration Notes
Users of OldProvider should migrate to [Alternative] for similar functionality.
```

---

## 🚀 Commit and Deploy Phase

### 1. Review Changes

```bash
# See what files changed
git status

# Review diffs
git diff

# Check for accidental commits of secrets
git diff | grep -i "api[_-]key.*=" | grep -v '""'
# Should return nothing!
```

### 2. Run Final Checks

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run tests (if any exist)
cargo test

# Build release
cargo build --release
```

### 3. Commit Changes

```bash
# Stage files
git add -A

# Create detailed commit message
git commit -m "$(cat <<'EOF'
Update providers and models based on [Date] research

Updated provider information, model names, and free tier details based on
current documentation and testing.

Changes:
- Updated Groq model names to current API (verified working)
- Updated DeepSeek free tier information (now unlimited)
- Added NewProvider with free tier (X requests/day)
- Updated Together AI pricing note ($5 deposit requirement)
- Removed OldProvider (service discontinued)
- Updated provider priority order for optimal routing

Model changes:
- Groq: Verified llama-3.3-70b-versatile still current
- DeepSeek: deepseek-chat confirmed working
- NewProvider: Added provider-model-name (chat, code workloads)

Documentation:
- Updated docs/providers-setup.md with current information
- Updated README.md provider comparison table
- Updated .config/template.toml with new providers

Testing:
- All providers tested with sample requests
- Health tracking verified
- Model catalog seeding confirmed
- Interactive setup wizard tested

Research sources:
- [List key URLs used for verification]

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
EOF
)"
```

### 4. Push Changes

```bash
# Push to main branch
git push origin main

# Or create PR if working on a branch
git checkout -b update-providers-$(date +%Y%m)
git push origin update-providers-$(date +%Y%m)
# Then create PR on GitHub
```

---

## 🤖 AI Assistant Prompt

Use this prompt when asking an AI assistant to help with updates:

```markdown
I need help updating AI providers and models for the freegin-ai project. Please:

1. **Research Phase**: Search for current information about these providers:
   - Groq (https://console.groq.com)
   - DeepSeek (https://platform.deepseek.com)
   - Together AI (https://api.together.xyz)
   - Google Gemini (https://ai.google.dev)
   - Hugging Face (https://huggingface.co)

   For each provider, find:
   - Current free tier limits (requests/day, tokens/minute)
   - Current model names (CRITICAL - verify these haven't changed)
   - Any pricing changes
   - Any new "free" models added

   Also search for:
   - New AI providers with generous free tiers
   - Community discussions about best free AI APIs
   - Recent provider shutdowns or changes

2. **Verification**: For each model name currently in the project, verify it still works:
   - Check official documentation
   - Look for GitHub issues mentioning "model not found"
   - Search for recent deprecation announcements

3. **Comparison**: Find providers we're NOT using that might be better:
   - Look for providers with better free tiers
   - Find providers with unique capabilities
   - Check for providers with higher rate limits

4. **Compilation**: Create a report with:
   - Current status of each provider (working? changed? deprecated?)
   - Model names that need updating
   - New providers to add
   - Providers to remove
   - Priority order recommendations

5. **Implementation Plan**: Based on your findings, list the specific code changes needed:
   - Which model names to update in src/catalog.rs
   - Which providers to add (with API format details)
   - Which providers to remove
   - Documentation updates needed

Please search the web for current information (prioritize sources from the last 3 months) and compile your findings in the format described in docs/update-providers-and-models.md.

Focus on: FREE TIER providers with generous limits. No paid-only providers.

Current project structure is at: /home/quagoo/freegin-ai
```

---

## 📊 Research Checklist

Before implementing changes, verify you have:

### For Each Current Provider:
- [ ] Verified free tier limits are still accurate
- [ ] Tested at least one model name works via API
- [ ] Checked for deprecation announcements (last 6 months)
- [ ] Read recent community feedback (Reddit, HN, GitHub)
- [ ] Confirmed API endpoint hasn't changed
- [ ] Verified authentication method unchanged

### For Each New Provider:
- [ ] Confirmed free tier exists and limits are reasonable (>100 req/day)
- [ ] Tested API works with sample request
- [ ] Verified API is stable (not beta/alpha)
- [ ] Checked community reputation
- [ ] Confirmed no surprise fees (like $5 deposit requirements)
- [ ] Determined API format (OpenAI-compatible vs custom)
- [ ] Found at least 2 recent sources confirming information

### For Model Names:
- [ ] Verified EACH model name in current catalog still works
- [ ] Found official list of available models
- [ ] Tested model names with actual API call (if possible)
- [ ] Checked for "model not found" errors in logs
- [ ] Cross-referenced with community discussions

---

## 🔄 Update Frequency

Recommended schedule:

- **Monthly**: Quick check of model names (5-10 minutes)
  - Run test requests to each provider
  - Check for any 404 errors
  - Quick scan of provider status pages

- **Quarterly**: Full provider review (2-4 hours)
  - Complete web research phase
  - Verify all information current
  - Look for new providers
  - Update documentation

- **As Needed**: When errors detected
  - 404 "model not found" → immediate model name update
  - Provider unavailable → check status and update
  - Community reports issues → investigate and verify

---

## ⚠️ Common Pitfalls

1. **Model Name Changes**: Providers frequently rename models. Always verify current names.

2. **Free Tier Changes**: "Free" can become "pay-per-use" overnight. Check pricing pages.

3. **Rate Limit Changes**: Limits can be reduced without notice. Monitor actual usage.

4. **API Endpoint Changes**: Providers sometimes change base URLs. Keep URLs current.

5. **Deposit Requirements**: Some "free" tiers require $5 deposit. Document these clearly.

6. **Deprecated Models**: Old model names may still "work" but route to different models. Verify behavior.

7. **Regional Restrictions**: Some providers have geographic limits on free tiers.

8. **Authentication Changes**: API key formats or header names can change.

---

## 📚 Additional Resources

- **Provider Documentation**: Always check official docs first
- **Community**: r/LocalLLaMA for latest provider discussions
- **Benchmarks**: https://artificialanalysis.ai for performance comparisons
- **Status Pages**: Check provider status pages for known issues
- **Release Notes**: Follow provider blogs for announcements

---

## ✅ Success Criteria

An update is successful when:

1. ✅ All existing providers still work with test requests
2. ✅ Model names are verified current via API testing
3. ✅ New providers (if any) successfully added and tested
4. ✅ Documentation accurately reflects current state
5. ✅ No secrets committed to git
6. ✅ All tests pass
7. ✅ `freegin-ai status` shows all providers healthy
8. ✅ Priority order makes sense based on current free tiers

---

**Last Updated**: [Update this when you run the process]
**Next Review**: [Set date for next quarterly review]

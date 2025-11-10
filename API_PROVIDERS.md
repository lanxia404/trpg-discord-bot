# API Provider Configuration Guide

This document explains how to configure and use different API providers with the TRPG Discord Bot, including OpenAI, OpenRouter, and other OpenAI-compatible APIs.

## Supported API Providers

The bot supports the following API providers:

1. **OpenAI** - Standard OpenAI API
2. **OpenRouter** - Unified interface for 280+ open and commercial models
3. **Anthropic** - Claude models (via OpenAI-compatible endpoints)
4. **Google** - Gemini models (via OpenAI-compatible endpoints)
5. **Custom** - Any OpenAI-compatible API endpoint

## Configuration

To configure an API provider, use the `/chat` command in your Discord server:

```
/chat add api_url:<api_endpoint> api_key:<your_api_key> model:<model_name>
```

The bot will automatically detect the provider based on the API URL:

- URLs containing `openrouter.ai` will be configured as OpenRouter
- URLs containing `anthropic` will be configured as Anthropic
- URLs containing `google` will be configured as Google
- All other URLs default to OpenAI

### OpenRouter Setup

1. Sign up at [OpenRouter.ai](https://openrouter.ai/)
2. Get your API key from the dashboard
3. Use the following command:

```
/chat add api_url:https://openrouter.ai/api/v1/chat/completions api_key:<your_openrouter_key> model:openai/gpt-4o
```

OpenRouter-specific features:
- Access to 280+ models including open and commercial models
- Automatic optimization for cost and speed
- Optional attribution headers added automatically

### OpenAI Setup

1. Get your OpenAI API key from the OpenAI dashboard
2. Use the following command:

```
/chat add api_url:https://api.openai.com/v1/chat/completions api_key:<your_openai_key> model:gpt-4o
```

### Custom API Endpoints

For other OpenAI-compatible services, use the base URL of the service:

```
/chat add api_url:<your_service_base_url> api_key:<your_api_key> model:<model_identifier>
```

## Provider-Specific Features

### OpenRouter
- **Attribution Headers**: The bot automatically adds optional headers for OpenRouter:
  - `HTTP-Referer`: Points to the bot's repository
  - `X-Title`: "TRPG Discord Bot"
- These help with OpenRouter's model ranking and attribution.

### Model Naming
- For OpenRouter, use full model identifiers like `openai/gpt-4o`, `anthropic/claude-3-sonnet`, etc.
- For OpenAI, use standard model names like `gpt-4o`, `gpt-3.5-turbo`, etc.
- If no model is specified when configuring an API, the bot will automatically select an appropriate default model based on the API provider

## Testing API Configuration

Use the following command to test your API configuration:

```
/chat add api_url:<api_endpoint> api_key:<your_api_key> model:<model_name>
```

The bot will attempt a test request and confirm if the configuration works.

## Managing API Settings

- `/chat remove` - Remove the current API configuration
- `/chat toggle` - Enable/disable API functionality
- `/chat list-models` - List available models from the API provider

## Using Environment Variables (OpenAI-Compatible Format)

Instead of providing API keys through Discord commands, you can set them in your `.env` file. 
These keys work with OpenAI-compatible APIs like OpenRouter, OpenAI, and other services that 
follow the OpenAI API format.

1. Add the appropriate variable to your `.env` file:
   - For OpenAI: `OPENAI_API_KEY=your_key_here`
   - For OpenRouter: `OPENROUTER_API_KEY=your_key_here`
   - For Anthropic (via OpenAI-compatible endpoints): `ANTHROPIC_API_KEY=your_key_here`
   - For Google (via OpenAI-compatible endpoints): `GOOGLE_API_KEY=your_key_here`
   - For Custom OpenAI-compatible APIs: `CUSTOM_API_KEY=your_key_here`

2. When using `/chat add api_url:<url> model:<model>`, omit the api_key parameter
   The bot will automatically use the key from the environment variables

3. This method keeps your API keys out of Discord command history

Note: The TRPG Discord Bot supports all APIs that follow the OpenAI API format, 
making it compatible with many different providers.

## Troubleshooting

1. **API requests failing**: Check your API key and endpoint URL
2. **Model not found**: Verify the model name format (especially for OpenRouter)
3. **Rate limiting**: Check your API provider's rate limits and usage
4. **Unsupported provider**: Ensure your API is compatible with OpenAI's format
5. **API key not found**: Make sure the correct environment variable is set in your .env file

## Security Note

- Never share your API keys publicly
- The bot can use API keys from environment variables (defined in .env file) or received via commands
- When using the `/chat add` command without specifying an API key, the bot will attempt to use keys from environment variables
- When specifying an API key via command, it's stored only in memory during the bot's runtime
- Only users with developer permissions can access API configuration commands

## 對話歷史功能

Bot 在處理 AI 請求時，會從數據庫中獲取最近的 100 條消息作為上下文，以提供更連貫的對話體驗。
use crate::utils::config::ConfigManager;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use log;
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    #[serde(default = "default_api_name")]
    pub name: String,
    pub api_url: String,
    pub api_key: Option<String>,
    pub model: String,
    pub enabled: bool,
    pub provider: ApiProvider,  // New field to identify API provider
}

fn default_api_name() -> String {
    "default".to_string()
}

// Helper function to get API key from environment variables based on provider
// These keys work with OpenAI-compatible APIs like OpenRouter, OpenAI, etc.
pub fn get_api_key_from_env(provider: &ApiProvider) -> Option<String> {
    match provider {
        ApiProvider::OpenAI => env::var("OPENAI_API_KEY").ok(),
        ApiProvider::OpenRouter => env::var("OPENROUTER_API_KEY").ok(),
        ApiProvider::Anthropic => env::var("ANTHROPIC_API_KEY").ok(),  // For Anthropic via OpenAI-compatible endpoints
        ApiProvider::Google => env::var("GOOGLE_API_KEY").ok(),        // For Google via OpenAI-compatible endpoints
        ApiProvider::Custom => {
            // For custom OpenAI-compatible APIs
            env::var("CUSTOM_API_KEY").ok()
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApiProvider {
    OpenAI,
    OpenRouter,
    Anthropic,
    Google,
    Custom,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            api_url: "https://api.openai.com/v1/chat/completions".to_string(),
            api_key: None,
            model: "gpt-3.5-turbo".to_string(),
            enabled: false,
            provider: ApiProvider::OpenAI,
        }
    }
}

#[derive(Debug)]
pub struct ApiManager {
    // 使用配置管理器而不是自己的HashMap
    pub config_manager: Arc<tokio::sync::Mutex<ConfigManager>>,
}

impl ApiManager {
    pub fn new(config_manager: Arc<tokio::sync::Mutex<ConfigManager>>) -> Self {
        Self {
            config_manager,
        }
    }

    pub async fn get_guild_config(&self, guild_id: u64) -> ApiConfig {
        self.config_manager.lock().await.get_guild_api_config(guild_id).await
    }

    pub async fn add_guild_config(&self, guild_id: u64, config: ApiConfig) {
        let _ = self.config_manager.lock().await.add_guild_api_config(guild_id, config).await;
    }

    pub async fn get_guild_configs(&self, guild_id: u64) -> std::collections::HashMap<String, ApiConfig> {
        self.config_manager.lock().await.get_guild_api_configs(guild_id).await
    }

    pub async fn remove_guild_config(&self, guild_id: u64, name: &str) -> bool {
        self.config_manager.lock().await.remove_guild_api_config(guild_id, name).await.unwrap_or(false)
    }

    pub async fn set_active_api(&self, guild_id: u64, name: &str) -> bool {
        self.config_manager.lock().await.set_active_api(guild_id, name).await.unwrap_or(false)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Choice {
    pub message: ChatMessage,
    pub index: u32,
    pub finish_reason: String,
}

pub async fn call_llm_api(
    api_url: &str,
    api_key: Option<&str>,
    request: &ChatCompletionRequest,
    provider: &ApiProvider,  // New parameter
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    log::info!("API 請求: URL={}, Model={}, Provider={:?}", api_url, request.model, provider);

    let client = reqwest::Client::new();

    // 構建請求 URL 根據不同提供商
    let (final_url, additional_headers) = build_request_params(api_url, provider);

    log::info!("最終 API 請求 URL: {}", final_url);

    let mut builder = client.post(&final_url);

    // 添加必要的 headers
    for (key, value) in additional_headers {
        builder = builder.header(key, value);
    }

    // 添加認證 header
    if let Some(key) = api_key {
        builder = builder.header("Authorization", format!("Bearer {}", key));
    }

    builder = builder.header("Content-Type", "application/json");

    log::debug!("API 請求內容: {:?}", request);

    let response = builder.json(request).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        log::error!("API 請求失敗: Status={}, Response={}, Model={}", status, error_text, request.model);
        return Err(format!("API request failed with status {}: {}", status, error_text).into());
    }

    // 盡可能解析響應，如果解析失敗，記錄詳細錯誤信息
    let response_text = response.text().await?;
    log::debug!("原始響應: {}", response_text);

    // 如果響應看起來像 HTML，則返回錯誤
    if response_text.starts_with("<!DOCTYPE html") || response_text.contains("<html") {
        log::error!("收到 HTML 響應而不是 JSON，API 端點 URL 可能不正確");
        return Err("收到 HTML 響應而不是 JSON，API 端點 URL 可能不正確".into());
    }

    // 嘗試解析 JSON 響應
    let json_value: serde_json::Value = match serde_json::from_str(&response_text) {
        Ok(val) => val,
        Err(e) => {
            log::error!("JSON 解析錯誤: {}，原始響應: {}", e, response_text);
            return Err(format!("JSON 解析失敗: {}", e).into());
        }
    };

    // 嘗試解析為預期的響應結構
    let completion_response: ChatCompletionResponse = match serde_json::from_value(json_value.clone()) {
        Ok(val) => val,
        Err(e) => {
            log::warn!("標準響應格式解析失敗: {}，嘗試通用解析", e);
            
            // 解析通用響應格式
            if let Some(choices_array) = json_value["choices"].as_array() {
                if let Some(first_choice) = choices_array.first() {
                    if let Some(message_obj) = first_choice["message"].as_object() {
                        if let Some(content) = message_obj["content"].as_str() {
                            log::info!("通用解析成功，回應長度: {}", content.len());
                            return Ok(content.to_string());
                        }
                    }
                }
            }
            
            log::error!("無法解析響應: {:?}", json_value);
            return Err("無法解析 API 響應".into());
        }
    };

    if let Some(choice) = completion_response.choices.first() {
        log::info!("API 回應成功: 回應長度={}", choice.message.content.len());
        Ok(choice.message.content.clone())
    } else {
        log::warn!("API 回應中沒有選擇: Model={}", request.model);
        Err("No response from LLM".into())
    }
}

// Helper function to build request parameters based on API provider
fn build_request_params(api_url: &str, provider: &ApiProvider) -> (String, Vec<(&'static str, String)>) {
    match provider {
        ApiProvider::OpenAI => {
            let final_url = if api_url.ends_with("/v1") && !api_url.contains("chat/completions") {
                format!("{}/chat/completions", api_url)
            } else {
                api_url.to_string()
            };
            (final_url, vec![])
        },
        ApiProvider::OpenRouter => {
            let final_url = if api_url.ends_with("/v1") && !api_url.contains("chat/completions") {
                format!("{}/chat/completions", api_url)
            } else if api_url == "https://api.openai.com/v1/chat/completions" {
                // Default to OpenRouter if using OpenAI default but configured as OpenRouter
                "https://openrouter.ai/api/v1/chat/completions".to_string()
            } else {
                api_url.to_string()
            };
            
            // Add optional attribution headers for OpenRouter
            let headers = vec![
                ("HTTP-Referer", "https://github.com/your-repo/trpg-discord-bot".to_string()),
                ("X-Title", "TRPG Discord Bot".to_string())
            ];
            
            (final_url, headers)
        },
        ApiProvider::Anthropic => {
            // Anthropic uses different format, but this function is for OpenAI-compatible APIs
            // So we return the original API URL with placeholder headers
            // Note: For full Anthropic support, we'd need a different implementation
            (api_url.to_string(), vec![])
        },
        ApiProvider::Google => {
            // Google also has different structure
            (api_url.to_string(), vec![])
        },
        ApiProvider::Custom => {
            // Custom endpoint with no specific modifications
            (api_url.to_string(), vec![])
        }
    }
}

// Helper function to get a default model based on provider
pub fn get_default_model_for_provider(provider: &ApiProvider) -> String {
    match provider {
        ApiProvider::OpenRouter => "google/gemma-2-9b-it".to_string(), // Free OpenRouter model
        ApiProvider::OpenAI => "gpt-3.5-turbo".to_string(),           // Standard OpenAI model
        ApiProvider::Anthropic => "claude-3-haiku-20240307".to_string(), // Anthropic free model
        ApiProvider::Google => "google/gemini-pro".to_string(),       // Google model
        ApiProvider::Custom => "gpt-3.5-turbo".to_string(),           // Default fallback
    }
}

pub async fn get_models_list(
    api_url: &str,
    api_key: Option<&str>,
    provider: &ApiProvider,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    log::info!("獲取模型列表: URL={}, Provider={:?}", api_url, provider);

    let client = reqwest::Client::new();

    // 構建模型列表 URL 根據不同提供商
    let (final_url, additional_headers) = build_models_list_params(api_url, provider);

    log::info!("最終模型列表 URL: {}", final_url);

    let mut builder = client.get(&final_url);

    // 添加必要的 headers
    for (key, value) in additional_headers {
        builder = builder.header(key, value);
    }

    if let Some(key) = api_key {
        builder = builder.header("Authorization", format!("Bearer {}", key));
    }

    let response = builder.send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        log::error!("獲取模型列表失敗: Status={}, Response={}", status, error_text);
        // 如果標準模型端點失敗，返回一個錯誤，讓前端處理其他方式
        return Err(format!("Failed to fetch models list: status {}, response: {}", status, error_text).into());
    }

    let response_text = response.text().await?;
    log::debug!("模型列表原始響應: {}", response_text);

    // 如果響應看起來像 HTML，則返回錯誤
    if response_text.starts_with("<!DOCTYPE html") || response_text.contains("<html") {
        log::error!("獲取模型列表時收到 HTML 響應而不是 JSON，API 端點 URL 可能不正確");
        return Err("收到 HTML 響應而不是 JSON，API 端點 URL 可能不正確".into());
    }

    let json: serde_json::Value = match serde_json::from_str(&response_text) {
        Ok(val) => val,
        Err(e) => {
            log::error!("模型列表 JSON 解析錯誤: {}，原始響應: {}", e, response_text);
            return Err(format!("JSON 解析失敗: {}", e).into());
        }
    };

    // 解析回應以獲取模型列表
    let mut models_list = Vec::new();
    if let Some(data) = json["data"].as_array() {
        for item in data {
            if let Some(model_id) = item["id"].as_str() {
                log::debug!("找到模型: {}", model_id);
                models_list.push(model_id.to_string());
            }
        }
    } else {
        // 某些服務可能有不同的響應結構，嘗試其他格式
        if let Some(array) = json.as_array() {
            // 如果響應是簡單的陣列，包含模型ID
            for item in array {
                if let Some(model_id) = item.get("id")
                    .or_else(|| item.get("model"))
                    .and_then(|v| v.as_str()) {
                    log::debug!("找到模型 (陣列格式): {}", model_id);
                    models_list.push(model_id.to_string());
                }
            }
        }
    }

    log::info!("模型列表獲取成功，共 {} 個模型", models_list.len());
    Ok(models_list)
}

// Helper function to build models list parameters based on API provider
fn build_models_list_params(api_url: &str, provider: &ApiProvider) -> (String, Vec<(&'static str, String)>) {
    match provider {
        ApiProvider::OpenAI => {
            let final_url = if api_url.ends_with("/v1") && !api_url.contains("models") {
                api_url.to_string() + "/models"
            } else if api_url.contains("chat/completions") {
                api_url.replace("chat/completions", "models")
            } else {
                api_url.rsplit_once('/')
                    .map(|(prefix, _)| format!("{}/models", prefix))
                    .unwrap_or_else(|| format!("{}/models", api_url))
            };
            (final_url, vec![])
        },
        ApiProvider::OpenRouter => {
            let final_url = if api_url.ends_with("/v1") && !api_url.contains("models") {
                api_url.to_string() + "/models"
            } else if api_url.contains("chat/completions") {
                api_url.replace("chat/completions", "models")
            } else {
                api_url.rsplit_once('/')
                    .map(|(prefix, _)| format!("{}/models", prefix))
                    .unwrap_or_else(|| format!("{}/models", api_url))
            };
            
            // Add optional attribution headers for OpenRouter
            let headers = vec![
                ("HTTP-Referer", "https://github.com/your-repo/trpg-discord-bot".to_string()),
                ("X-Title", "TRPG Discord Bot".to_string())
            ];
            
            (final_url, headers)
        },
        ApiProvider::Anthropic => {
            // Anthropic doesn't have a standard models endpoint like OpenAI
            // Return the original URL with no modifications
            (api_url.to_string(), vec![])
        },
        ApiProvider::Google => {
            // Google also has different structure
            (api_url.to_string(), vec![])
        },
        ApiProvider::Custom => {
            // Custom endpoint with no specific modifications
            (api_url.to_string(), vec![])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_api_manager_creation() {
        let _api_manager = ApiManager::new();
        // 測試默認配置是否正確
        let config = ApiConfig::default();
        assert_eq!(config.model, "gpt-3.5-turbo");
        assert_eq!(config.provider, ApiProvider::OpenAI);
    }

    #[test]
    fn test_build_request_params_openai() {
        let (url, headers) = build_request_params("https://api.openai.com/v1/chat/completions", &ApiProvider::OpenAI);
        assert_eq!(url, "https://api.openai.com/v1/chat/completions");
        assert!(headers.is_empty());
    }

    #[test]
    fn test_build_request_params_openrouter() {
        let (url, headers) = build_request_params("https://openrouter.ai/api/v1/chat/completions", &ApiProvider::OpenRouter);
        assert_eq!(url, "https://openrouter.ai/api/v1/chat/completions");
        assert!(!headers.is_empty()); // Should have attribution headers
    }
}
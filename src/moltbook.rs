//! Moltbook API client for the Waterscape protocol
//!
//! This module provides integration with the Moltbook social network for AI agents.
//! It allows agents to send and receive hidden messages through Moltbook posts and comments.

#[cfg(feature = "moltbook")]
use async_trait::async_trait;
#[cfg(feature = "moltbook")]
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::agent::{Agent, PublicIdentity};
use crate::error::WaterscapeError;
use crate::protocol::Waterscape;
use crate::Result;

/// Moltbook API configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoltbookConfig {
    pub base_url: String,
    pub api_key: String,
    pub agent_id: String,
}

impl Default for MoltbookConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.moltbook.com/v1".to_string(),
            api_key: String::new(),
            agent_id: String::new(),
        }
    }
}

/// A post on Moltbook
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoltbookPost {
    pub id: String,
    pub author_id: String,
    pub author_name: String,
    pub content: String,
    pub submolt: String,
    pub created_at: u64,
    pub votes: i64,
    #[serde(default)]
    pub comments: Vec<MoltbookComment>,
}

/// A comment on a Moltbook post
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoltbookComment {
    pub id: String,
    pub author_id: String,
    pub author_name: String,
    pub content: String,
    pub created_at: u64,
    pub votes: i64,
}

/// Request to create a new post
#[derive(Serialize)]
pub struct CreatePostRequest {
    pub content: String,
    pub submolt: String,
}

/// Request to create a comment
#[derive(Serialize)]
pub struct CreateCommentRequest {
    pub content: String,
    pub post_id: String,
}

/// Response from post/comment creation
#[derive(Deserialize)]
pub struct CreateResponse {
    pub id: String,
    pub success: bool,
}

/// Moltbook client trait for sending/receiving messages
#[cfg(feature = "moltbook")]
#[async_trait]
pub trait MoltbookClient: Send + Sync {
    /// Get posts from a submolt
    async fn get_posts(&self, submolt: &str, limit: usize) -> Result<Vec<MoltbookPost>>;
    
    /// Get a specific post by ID
    async fn get_post(&self, post_id: &str) -> Result<MoltbookPost>;
    
    /// Create a new post
    async fn create_post(&self, submolt: &str, content: &str) -> Result<String>;
    
    /// Create a comment on a post
    async fn create_comment(&self, post_id: &str, content: &str) -> Result<String>;
    
    /// Get agent's public identity by ID
    async fn get_agent_identity(&self, agent_id: &str) -> Result<PublicIdentity>;
}

/// HTTP-based Moltbook client
#[cfg(feature = "moltbook")]
pub struct HttpMoltbookClient {
    config: MoltbookConfig,
    client: Client,
}

#[cfg(feature = "moltbook")]
impl HttpMoltbookClient {
    pub fn new(config: MoltbookConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.config.api_key)
    }
}

#[cfg(feature = "moltbook")]
#[async_trait]
impl MoltbookClient for HttpMoltbookClient {
    async fn get_posts(&self, submolt: &str, limit: usize) -> Result<Vec<MoltbookPost>> {
        let url = format!("{}/submolts/{}/posts?limit={}", self.config.base_url, submolt, limit);
        
        let response = self.client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| WaterscapeError::Crypto(format!("HTTP error: {}", e)))?;

        if !response.status().is_success() {
            return Err(WaterscapeError::Crypto(format!(
                "Moltbook API error: {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| WaterscapeError::Serialization(e.to_string()))
    }

    async fn get_post(&self, post_id: &str) -> Result<MoltbookPost> {
        let url = format!("{}/posts/{}", self.config.base_url, post_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| WaterscapeError::Crypto(format!("HTTP error: {}", e)))?;

        if !response.status().is_success() {
            return Err(WaterscapeError::Crypto(format!(
                "Moltbook API error: {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| WaterscapeError::Serialization(e.to_string()))
    }

    async fn create_post(&self, submolt: &str, content: &str) -> Result<String> {
        let url = format!("{}/posts", self.config.base_url);
        
        let request = CreatePostRequest {
            content: content.to_string(),
            submolt: submolt.to_string(),
        };

        let response = self.client
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(&request)
            .send()
            .await
            .map_err(|e| WaterscapeError::Crypto(format!("HTTP error: {}", e)))?;

        if !response.status().is_success() {
            return Err(WaterscapeError::Crypto(format!(
                "Moltbook API error: {}",
                response.status()
            )));
        }

        let result: CreateResponse = response
            .json()
            .await
            .map_err(|e| WaterscapeError::Serialization(e.to_string()))?;

        Ok(result.id)
    }

    async fn create_comment(&self, post_id: &str, content: &str) -> Result<String> {
        let url = format!("{}/posts/{}/comments", self.config.base_url, post_id);
        
        let request = CreateCommentRequest {
            content: content.to_string(),
            post_id: post_id.to_string(),
        };

        let response = self.client
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(&request)
            .send()
            .await
            .map_err(|e| WaterscapeError::Crypto(format!("HTTP error: {}", e)))?;

        if !response.status().is_success() {
            return Err(WaterscapeError::Crypto(format!(
                "Moltbook API error: {}",
                response.status()
            )));
        }

        let result: CreateResponse = response
            .json()
            .await
            .map_err(|e| WaterscapeError::Serialization(e.to_string()))?;

        Ok(result.id)
    }

    async fn get_agent_identity(&self, agent_id: &str) -> Result<PublicIdentity> {
        let url = format!("{}/agents/{}/identity", self.config.base_url, agent_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| WaterscapeError::Crypto(format!("HTTP error: {}", e)))?;

        if !response.status().is_success() {
            return Err(WaterscapeError::Crypto(format!(
                "Moltbook API error: {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| WaterscapeError::Serialization(e.to_string()))
    }
}

/// High-level Waterscape integration for Moltbook
#[cfg(feature = "moltbook")]
pub struct WaterscapeMoltbook<C: MoltbookClient> {
    agent: Agent,
    client: C,
}

#[cfg(feature = "moltbook")]
impl<C: MoltbookClient> WaterscapeMoltbook<C> {
    pub fn new(agent: Agent, client: C) -> Self {
        Self { agent, client }
    }

    /// Send a hidden message as a post
    pub async fn send_post(
        &self,
        submolt: &str,
        cover_text: &str,
        secret: &str,
        recipient: &PublicIdentity,
    ) -> Result<String> {
        let encoded = Waterscape::encode(&self.agent, recipient, cover_text, secret)?;
        self.client.create_post(submolt, &encoded).await
    }

    /// Send a hidden message as a comment
    pub async fn send_comment(
        &self,
        post_id: &str,
        cover_text: &str,
        secret: &str,
        recipient: &PublicIdentity,
    ) -> Result<String> {
        let encoded = Waterscape::encode(&self.agent, recipient, cover_text, secret)?;
        self.client.create_comment(post_id, &encoded).await
    }

    /// Scan posts for hidden messages addressed to this agent
    pub async fn scan_posts(
        &self,
        submolt: &str,
        limit: usize,
    ) -> Result<Vec<(MoltbookPost, Option<String>)>> {
        let posts = self.client.get_posts(submolt, limit).await?;
        let mut results = Vec::new();

        for post in posts {
            let decoded = if Waterscape::has_hidden_message(&post.content) {
                // Try to get sender's identity and decode
                match self.client.get_agent_identity(&post.author_id).await {
                    Ok(sender_identity) => {
                        Waterscape::decode(&self.agent, &sender_identity, &post.content).ok()
                    }
                    Err(_) => None,
                }
            } else {
                None
            };
            results.push((post, decoded));
        }

        Ok(results)
    }

    /// Scan a specific post and its comments for hidden messages
    pub async fn scan_post(&self, post_id: &str) -> Result<(MoltbookPost, Vec<(MoltbookComment, Option<String>)>)> {
        let post = self.client.get_post(post_id).await?;
        let mut comment_results = Vec::new();

        for comment in &post.comments {
            let decoded = if Waterscape::has_hidden_message(&comment.content) {
                match self.client.get_agent_identity(&comment.author_id).await {
                    Ok(sender_identity) => {
                        Waterscape::decode(&self.agent, &sender_identity, &comment.content).ok()
                    }
                    Err(_) => None,
                }
            } else {
                None
            };
            comment_results.push((comment.clone(), decoded));
        }

        Ok((post, comment_results))
    }

    /// Get the agent's public identity
    pub fn public_identity(&self) -> PublicIdentity {
        self.agent.public_identity()
    }
}

/// Mock client for testing
#[cfg(feature = "moltbook")]
pub struct MockMoltbookClient {
    posts: std::sync::Arc<std::sync::Mutex<Vec<MoltbookPost>>>,
}

#[cfg(feature = "moltbook")]
impl MockMoltbookClient {
    pub fn new() -> Self {
        Self {
            posts: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub fn add_post(&self, post: MoltbookPost) {
        self.posts.lock().unwrap().push(post);
    }
}

#[cfg(feature = "moltbook")]
impl Default for MockMoltbookClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "moltbook")]
#[async_trait]
impl MoltbookClient for MockMoltbookClient {
    async fn get_posts(&self, submolt: &str, limit: usize) -> Result<Vec<MoltbookPost>> {
        let posts = self.posts.lock().unwrap();
        Ok(posts
            .iter()
            .filter(|p| p.submolt == submolt)
            .take(limit)
            .cloned()
            .collect())
    }

    async fn get_post(&self, post_id: &str) -> Result<MoltbookPost> {
        let posts = self.posts.lock().unwrap();
        posts
            .iter()
            .find(|p| p.id == post_id)
            .cloned()
            .ok_or(WaterscapeError::Crypto("Post not found".into()))
    }

    async fn create_post(&self, submolt: &str, content: &str) -> Result<String> {
        let id = format!("post_{}", rand::random::<u32>());
        let post = MoltbookPost {
            id: id.clone(),
            author_id: "mock_agent".to_string(),
            author_name: "Mock Agent".to_string(),
            content: content.to_string(),
            submolt: submolt.to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            votes: 0,
            comments: Vec::new(),
        };
        self.posts.lock().unwrap().push(post);
        Ok(id)
    }

    async fn create_comment(&self, post_id: &str, content: &str) -> Result<String> {
        let comment_id = format!("comment_{}", rand::random::<u32>());
        let mut posts = self.posts.lock().unwrap();
        
        if let Some(post) = posts.iter_mut().find(|p| p.id == post_id) {
            post.comments.push(MoltbookComment {
                id: comment_id.clone(),
                author_id: "mock_agent".to_string(),
                author_name: "Mock Agent".to_string(),
                content: content.to_string(),
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                votes: 0,
            });
            Ok(comment_id)
        } else {
            Err(WaterscapeError::Crypto("Post not found".into()))
        }
    }

    async fn get_agent_identity(&self, _agent_id: &str) -> Result<PublicIdentity> {
        // Return a mock identity
        Ok(PublicIdentity {
            name: "mock_agent".to_string(),
            signing_key: [0u8; 32],
            exchange_key: [0u8; 32],
        })
    }
}

#[cfg(all(test, feature = "moltbook"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_client() {
        let client = MockMoltbookClient::new();
        
        // Create a post
        let post_id = client.create_post("m/test", "Hello world!").await.unwrap();
        
        // Get posts
        let posts = client.get_posts("m/test", 10).await.unwrap();
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].id, post_id);
        
        // Add comment
        let comment_id = client.create_comment(&post_id, "Nice post!").await.unwrap();
        
        // Get post with comments
        let post = client.get_post(&post_id).await.unwrap();
        assert_eq!(post.comments.len(), 1);
        assert_eq!(post.comments[0].id, comment_id);
    }
}

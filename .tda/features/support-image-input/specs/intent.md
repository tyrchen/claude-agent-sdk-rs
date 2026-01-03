# Feature Intent

Current Limitation

  The Rust SDK does not yet support images in user prompts. The query() and query_with_session() methods only accept impl Into<String>:

  // src/client.rs:230-238
  let user_message = serde_json::json!({
      "type": "user",
      "message": {
          "role": "user",
          "content": prompt_str  // Only text supported
      },
      ...
  });

  What IS Supported

  Tool results can return images via ToolResultContent::Image in src/types/mcp.rs:102-118:

  pub enum ToolResultContent {
      Text { text: String },
      Image {
          data: String,      // Base64-encoded
          mime_type: String, // e.g., "image/png"
      },
  }

  To Add Image Support in User Prompts

  You would need to:

  1. Add an ImageBlock variant to ContentBlock enum in src/types/messages.rs:
  pub enum ContentBlock {
      Text(TextBlock),
      Image(ImageBlock),  // Add this
      // ...
  }
  2. Create the ImageBlock struct:
  pub struct ImageBlock {
      pub source: ImageSource,
  }

  pub struct ImageSource {
      pub type_: String,    // "base64"
      pub media_type: String, // "image/png", "image/jpeg", etc.
      pub data: String,     // Base64-encoded image data
  }
  3. Extend the client API to accept content blocks, not just strings, in query() methods.
  4. Update message serialization in client.rs to handle structured content.

  The Python SDK already supports this pattern, so you can reference vendors/claude-agent-sdk-python for the expected API shape. Think this hard and build a concrete solution

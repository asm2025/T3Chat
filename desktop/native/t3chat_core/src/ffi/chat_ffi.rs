use crate::api::{chat, chats};
use crate::ffi::types::{
    FfiChat, FfiChatChunk, FfiChatListResponse, FfiChatWithMessages, FfiError,
};
use crate::ffi::auth_ffi::get_client;
use flutter_rust_bridge::frb;
use futures::StreamExt;

#[frb(sync)]
pub fn ffi_list_chats(page: Option<u64>, page_size: Option<u64>) -> Result<FfiChatListResponse, FfiError> {
    let client = get_client()?;
    let rt = tokio::runtime::Runtime::new().map_err(|e| FfiError::Unknown(e.to_string()))?;
    rt.block_on(async {
        chats::list_chats(&client, page, page_size)
            .await
            .map(|resp| FfiChatListResponse {
                data: resp.data.into_iter().map(FfiChat::from).collect(),
                total: resp.total,
            })
            .map_err(FfiError::from)
    })
}

#[frb(sync)]
pub fn ffi_create_chat(
    title: Option<String>,
    model_provider: String,
    model_id: String,
) -> Result<FfiChat, FfiError> {
    let client = get_client()?;
    let rt = tokio::runtime::Runtime::new().map_err(|e| FfiError::Unknown(e.to_string()))?;
    rt.block_on(async {
        let request = chats::CreateChatRequest {
            title,
            model_provider,
            model_id,
        };
        chats::create_chat(&client, request)
            .await
            .map(FfiChat::from)
            .map_err(FfiError::from)
    })
}

#[frb(sync)]
pub fn ffi_get_chat(chat_id: String) -> Result<FfiChatWithMessages, FfiError> {
    let client = get_client()?;
    let rt = tokio::runtime::Runtime::new().map_err(|e| FfiError::Unknown(e.to_string()))?;
    rt.block_on(async {
        chats::get_chat(&client, &chat_id)
            .await
            .map(|cwm| FfiChatWithMessages {
                chat: FfiChat::from(cwm.chat),
                messages: cwm.messages.into_iter().map(crate::ffi::types::FfiMessage::from).collect(),
            })
            .map_err(FfiError::from)
    })
}

#[frb(sync)]
pub fn ffi_update_chat(chat_id: String, title: Option<String>) -> Result<FfiChat, FfiError> {
    let client = get_client()?;
    let rt = tokio::runtime::Runtime::new().map_err(|e| FfiError::Unknown(e.to_string()))?;
    rt.block_on(async {
        let request = chats::UpdateChatRequest { title };
        chats::update_chat(&client, &chat_id, request)
            .await
            .map(FfiChat::from)
            .map_err(FfiError::from)
    })
}

#[frb(sync)]
pub fn ffi_delete_chat(chat_id: String) -> Result<(), FfiError> {
    let client = get_client()?;
    let rt = tokio::runtime::Runtime::new().map_err(|e| FfiError::Unknown(e.to_string()))?;
    rt.block_on(async { chats::delete_chat(&client, &chat_id).await.map_err(FfiError::from) })
}

// Note: Streaming is complex with flutter_rust_bridge sync functions
// For now, we'll collect all chunks and return them
// A proper async stream implementation would require async FFI functions
#[frb(sync)]
pub fn ffi_stream_chat(
    chat_id: String,
    message: String,
    model_provider: String,
    model_id: String,
) -> Result<Vec<FfiChatChunk>, FfiError> {
    let client = get_client()?;
    let rt = tokio::runtime::Runtime::new().map_err(|e| FfiError::Unknown(e.to_string()))?;
    rt.block_on(async {
        let request = chat::ChatRequest {
            chat_id,
            message,
            model_provider,
            model_id,
            model_parameters: None,
            feature_flags: None,
            system_message: None,
            stream: true,
        };
        
        let mut stream = chat::stream_message(&client, request).await?;
        let mut result = Vec::new();
        
        // Collect all chunks
        use futures::StreamExt;
        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    result.push(FfiChatChunk {
                        delta: chunk.delta,
                        done: chunk.done,
                    });
                    if chunk.done {
                        break;
                    }
                }
                Err(e) => return Err(FfiError::from(e)),
            }
        }
        
        Ok(result)
    })
}


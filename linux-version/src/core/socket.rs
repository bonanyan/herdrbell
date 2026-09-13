use std::path::Path;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

use crate::wire::{
    self, AgentInfo, AgentListResult, HerdrEventEnvelope, HerdrFrame, PingResult, SnapshotResult,
};

#[derive(Debug, thiserror::Error)]
pub enum SocketError {
    #[error("connect failed: {0}")]
    ConnectFailed(String),
    #[error("write failed: {0}")]
    WriteFailed(String),
    #[error("read failed: {0}")]
    ReadFailed(String),
    #[error("connection closed")]
    Closed,
    #[error("unexpected frame on socket")]
    UnexpectedFrame,
    #[error("response missing result")]
    MissingResult,
    #[error("herdr error {code}: {message}")]
    ServerError { code: String, message: String },
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type SocketResult<T> = Result<T, SocketError>;

pub struct HerdrSocket {
    path: String,
}

impl HerdrSocket {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub async fn ping(&self) -> SocketResult<PingResult> {
        self.request_typed::<PingResult>("ping", serde_json::json!({}))
            .await
    }

    pub async fn session_snapshot(&self) -> SocketResult<wire::SessionSnapshot> {
        let result = self
            .request_typed::<SnapshotResult>("session.snapshot", serde_json::json!({}))
            .await?;
        Ok(result.snapshot)
    }

    pub async fn agent_list(&self) -> SocketResult<Vec<AgentInfo>> {
        let result = self
            .request_typed::<AgentListResult>("agent.list", serde_json::json!({}))
            .await?;
        Ok(result.agents)
    }

    pub async fn agent_focus(&self, target: &str) -> SocketResult<()> {
        self.request("agent.focus", serde_json::json!({"target": target}))
            .await?;
        Ok(())
    }

    pub async fn request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> SocketResult<Option<serde_json::Value>> {
        let id = format!("req_{}", uuid::Uuid::new_v4());
        let request_line = wire::encode_request(&id, method, params);

        let stream = UnixStream::connect(Path::new(&self.path))
            .await
            .map_err(|e| SocketError::ConnectFailed(e.to_string()))?;

        let timeout = std::time::Duration::from_secs(10);
        let result = tokio::time::timeout(timeout, Self::do_request(stream, request_line))
            .await
            .map_err(|_| SocketError::ReadFailed("request timed out".into()))??;

        Ok(result)
    }

    async fn do_request(
        stream: UnixStream,
        request_line: Vec<u8>,
    ) -> SocketResult<Option<serde_json::Value>> {
        let (reader, mut writer) = stream.into_split();
        writer
            .write_all(&request_line)
            .await
            .map_err(|e| SocketError::WriteFailed(e.to_string()))?;

        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();
        buf_reader
            .read_line(&mut line)
            .await
            .map_err(|e| SocketError::ReadFailed(e.to_string()))?;

        if line.is_empty() {
            return Err(SocketError::Closed);
        }

        let frame = wire::parse_frame(line.as_bytes())?;
        match frame {
            HerdrFrame::Response(response) => {
                if let Some(error) = response.error {
                    return Err(SocketError::ServerError {
                        code: error.code,
                        message: error.message,
                    });
                }
                Ok(response.result)
            }
            HerdrFrame::Event(_) => Err(SocketError::UnexpectedFrame),
        }
    }

    async fn request_typed<T: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> SocketResult<T> {
        let result = self
            .request(method, params)
            .await?
            .ok_or(SocketError::MissingResult)?;
        wire::decode_result(&result).map_err(SocketError::Json)
    }

    pub async fn subscribe(
        &self,
        subscriptions: Vec<serde_json::Value>,
    ) -> SocketResult<tokio::sync::mpsc::Receiver<HerdrEventEnvelope>> {
        let stream = UnixStream::connect(Path::new(&self.path))
            .await
            .map_err(|e| SocketError::ConnectFailed(e.to_string()))?;

        let (reader, mut writer) = stream.into_split();

        let request_line = wire::encode_request(
            "req_sub",
            "events.subscribe",
            serde_json::json!({"subscriptions": subscriptions}),
        );
        writer
            .write_all(&request_line)
            .await
            .map_err(|e| SocketError::WriteFailed(e.to_string()))?;

        let mut buf_reader = BufReader::new(reader);
        let mut first_line = String::new();
        buf_reader
            .read_line(&mut first_line)
            .await
            .map_err(|e| SocketError::ReadFailed(e.to_string()))?;

        if first_line.is_empty() {
            return Err(SocketError::Closed);
        }

        let frame = wire::parse_frame(first_line.as_bytes())?;
        match frame {
            HerdrFrame::Response(response) => {
                if let Some(error) = response.error {
                    return Err(SocketError::ServerError {
                        code: error.code,
                        message: error.message,
                    });
                }
            }
            HerdrFrame::Event(_) => return Err(SocketError::UnexpectedFrame),
        }

        let (tx, rx) = tokio::sync::mpsc::channel(256);
        tokio::spawn(async move {
            let mut line_buf = String::new();
            loop {
                line_buf.clear();
                match buf_reader.read_line(&mut line_buf).await {
                    Ok(0) => break,
                    Ok(_) => {
                        if let Ok(HerdrFrame::Event(event)) = wire::parse_frame(line_buf.as_bytes())
                        {
                            if tx.send(event).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(rx)
    }
}

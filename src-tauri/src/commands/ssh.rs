use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use tokio::process::Command;

#[derive(Debug, Deserialize)]
pub struct TestConnectionRequest {
    pub host: String,
    pub username: String,
    pub port: Option<u16>,
    /// 例: "C:\\Users\\you\\.ssh\\id_ed25519" or "/home/you/.ssh/id_ed25519"
    pub private_key_path: Option<String>,
    /// known_hosts初回確認の挙動
    /// - true: accept-new（初回だけ自動受理）※ OpenSSH 7.6+ で有効
    /// - false: 既定（対話が必要になる可能性）
    pub accept_new_host_key: Option<bool>,
    /// タイムアウト秒（未指定は10秒）
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct TestConnectionResponse {
    pub ok: bool,
    pub message: String,
}

#[tauri::command]
pub async fn test_connection(payload: TestConnectionRequest) -> Result<TestConnectionResponse, String> {
    match test_connection_impl(payload).await {
        Ok(v) => Ok(v),
        Err(e) => Err(format!("{:#}", e)),
    }
}

async fn test_connection_impl(payload: TestConnectionRequest) -> Result<TestConnectionResponse> {
    let port = payload.port.unwrap_or(22);
    let timeout = payload.timeout_secs.unwrap_or(10);

    // ssh -p 22 user@host hostname
    let mut cmd = Command::new("ssh");

    // タイムアウト（OpenSSHのオプション）
    cmd.args(["-o", &format!("ConnectTimeout={}", timeout)]);

    // 初回known_hosts対話で止まらないように（任意）
    if payload.accept_new_host_key.unwrap_or(false) {
        cmd.args(["-o", "StrictHostKeyChecking=accept-new"]);
    }

    // 鍵ファイル指定（任意）
    if let Some(key_path) = payload.private_key_path.as_deref() {
        cmd.args(["-i", key_path]);
    }

    cmd.args(["-p", &port.to_string()]);
    cmd.arg(format!("{}@{}", payload.username, payload.host));

    // 実行するリモートコマンド（最小）
    cmd.arg("hostname");

    let out = cmd.output().await.context("failed to spawn ssh command")?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();

        // どちらも空なら一般エラー
        let msg = if !stderr.is_empty() { stderr } else { stdout };
        return Err(anyhow!("ssh failed: {}", msg));
    }

    let hostname = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if hostname.is_empty() {
        return Err(anyhow!("ssh succeeded but stdout is empty"));
    }

    Ok(TestConnectionResponse {
        ok: true,
        message: format!("Connected. Hostname: {}", hostname),
    })
}
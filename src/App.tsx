import "./App.css";
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type TestConnectionResponse = {
  ok: boolean;
  message: string;
};

type TestConnectionRequest = {
  host: string;
  username: string;
  port?: number;
  private_key_path?: string;
  accept_new_host_key?: boolean;
  timeout_secs?: number;
};

export default function App() {
  const [result, setResult] = useState<TestConnectionResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    (async () => {
      try {
        const payload: TestConnectionRequest = {
          host: "192.168.1.21",
          username: "user-admin",
          port: 22,
          accept_new_host_key: true,
          timeout_secs: 10,

          // 既定鍵で入れるなら不要（sshが ~/.ssh/config や既定鍵を使う）
          // 鍵を明示したいなら入れる：
          private_key_path: "C:\\Users\\sorai\\.ssh\\id_ed25519_server",
        };

        const res = await invoke<TestConnectionResponse>("test_connection", { payload });
        if (!cancelled) setResult(res);
      } catch (e) {
        if (!cancelled) setError(String(e));
      }
    })();

    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <div style={{ padding: 16, fontFamily: "sans-serif" }}>
      <h1>Connection Test</h1>

      {error && <pre style={{ whiteSpace: "pre-wrap" }}>Error: {error}</pre>}
      {!error && !result && <p>Testing...</p>}
      {result && <pre style={{ whiteSpace: "pre-wrap" }}>{JSON.stringify(result, null, 2)}</pre>}
    </div>
  );
}
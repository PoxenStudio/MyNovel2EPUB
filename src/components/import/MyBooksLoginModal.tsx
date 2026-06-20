import { useState } from "react";

const LAST_HOST_KEY = "mynovel2epub-mybooks-last-host";
const LAST_USERNAME_KEY = "mynovel2epub-mybooks-last-username";

interface Props {
  onLogin: (host: string, username: string, password: string) => Promise<void>;
  onClose: () => void;
}

export function MyBooksLoginModal({ onLogin, onClose }: Props) {
  const [host, setHost] = useState(() => localStorage.getItem(LAST_HOST_KEY) ?? "");
  const [username, setUsername] = useState(
    () => localStorage.getItem(LAST_USERNAME_KEY) ?? "",
  );
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    setIsLoading(true);
    localStorage.setItem(LAST_HOST_KEY, host);
    localStorage.setItem(LAST_USERNAME_KEY, username);
    try {
      await onLogin(host, username, password);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  }

  return (
    <div className="modal modal-open">
      <div className="modal-box">
        <h3 className="text-lg font-bold">登录 MyBooks 书库</h3>
        <form onSubmit={handleSubmit} className="mt-4 flex flex-col items-center gap-3">
          <label className="form-control w-full max-w-sm text-left">
            <span className="label-text">Host</span>
            <input
              type="text"
              className="input input-bordered w-full"
              placeholder="https://mybooks.example.com"
              value={host}
              onChange={(e) => setHost(e.target.value)}
              required
            />
          </label>
          <label className="form-control w-full max-w-sm text-left">
            <span className="label-text">用户名</span>
            <input
              type="text"
              className="input input-bordered w-full"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              required
            />
          </label>
          <label className="form-control w-full max-w-sm text-left">
            <span className="label-text">密码</span>
            <input
              type="password"
              className="input input-bordered w-full"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
            />
          </label>
          {error && <div className="alert alert-error w-full max-w-sm text-sm">{error}</div>}
          <div className="modal-action w-full max-w-sm">
            <button type="button" className="btn" onClick={onClose}>
              取消
            </button>
            <button type="submit" className="btn btn-primary" disabled={isLoading}>
              {isLoading ? "登录中..." : "登录"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

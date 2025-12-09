import { useState } from "react";
import { matrixService } from "../services/matrixService";


interface LoginProps {
  onLoginSuccess: (userId: string) => void;
}

export function Login({ onLoginSuccess }: LoginProps) {
  const [homeserver, setHomeserver] = useState("https://matrix.org");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState("");
  const [status, setStatus] = useState("");

  async function handleLogin(e: React.FormEvent) {
    e.preventDefault();
    setIsLoading(true);
    setError("");
    setStatus("Connecting to homeserver...");

    try {
      const response = await matrixService.login(homeserver, username, password);
      
      if (response.success) {
        setPassword("");
        onLoginSuccess(response.user_id);
      }
    } catch (error) {
      setError(String(error));
      setStatus("");
    } finally {
      setIsLoading(false);
    }
  }

  return (
    <div className="login-container">
      <div className="login-box">
        <div className="login-header">
          <h1>Matrix Client</h1>
          <p>Sign in to your Matrix account</p>
        </div>

        <form onSubmit={handleLogin} className="login-form">
          <div className="form-group">
            <label htmlFor="homeserver">Homeserver</label>
            <input
              id="homeserver"
              type="text"
              placeholder="https://matrix.org"
              value={homeserver}
              onChange={(e) => setHomeserver(e.target.value)}
              disabled={isLoading}
              required
            />
          </div>

          <div className="form-group">
            <label htmlFor="username">Username</label>
            <input
              id="username"
              type="text"
              placeholder="@username:matrix.org"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              disabled={isLoading}
              required
              autoComplete="username"
            />
          </div>

          <div className="form-group">
            <label htmlFor="password">Password</label>
            <input
              id="password"
              type="password"
              placeholder="Enter your password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              disabled={isLoading}
              required
              autoComplete="current-password"
            />
          </div>

          {error && <div className="error-message">{error}</div>}
          {status && !error && <div className="info-message">{status}</div>}

          <button type="submit" className="login-button" disabled={isLoading}>
            {isLoading ? "Signing in..." : "Sign In"}
          </button>
        </form>

        <div className="login-footer">
          <p>Don't have an account? Register on your homeserver</p>
          <p className="supportive-message">
            💙 You're not alone. Connecting with others is a sign of strength, not weakness.
          </p>
        </div>
      </div>
    </div>
  );
}

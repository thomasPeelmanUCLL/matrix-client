import { useState } from "react";
import { matrixService } from "../services/matrixService";

interface VerificationDialogProps {
  onClose: () => void;
  onVerified: () => void;
}

export function VerificationDialog({ onClose, onVerified }: VerificationDialogProps) {
  const [step, setStep] = useState<"request" | "emoji" | "waiting" | "recovery">("request");
  const [emoji, setEmoji] = useState<[string, string][]>([]);
  const [error, setError] = useState("");
  const [status, setStatus] = useState("");
  const [recoveryKey, setRecoveryKey] = useState("");

  async function startVerification() {
    try {
      setStatus("Requesting verification...");
      await matrixService.requestVerification();
      setStep("waiting");
      setStatus("Check your other device for verification request");
      
      // Poll for emoji
      pollForEmoji();
    } catch (e) {
      setError(String(e));
    }
  }

  async function startVerificationAlternate() {
    setStep("recovery");
    setError("");
    setStatus("");
  }

async function submitRecoveryKey() {
  if (!recoveryKey.trim()) {
    setError("Please enter your recovery key");
    return;
  }

  try {
    setStatus("Verifying recovery key...");
    setError("");
    
    await matrixService.requestRecoveryKeyVerification(recoveryKey.trim());
    
    setStatus("Recovery key verified! Syncing encryption keys...");
    
    // Give more time for keys to be imported and processed
    await new Promise(resolve => setTimeout(resolve, 3000));
    
    onVerified();
  } catch (e) {
    setError(String(e));
    setStatus("");
  }
}
  async function pollForEmoji() {
    const maxAttempts = 60;
    let attempts = 0;

    const interval = setInterval(async () => {
      attempts++;
      
      try {
        const emojiList = await matrixService.getVerificationEmoji();
        if (emojiList && emojiList.length > 0) {
          console.log("Got emoji!", emojiList);
          setEmoji(emojiList);
          setStep("emoji");
          setStatus("");
          clearInterval(interval);
        }
      } catch (e) {
        console.log("Polling for emoji:", String(e));
      }

      if (attempts >= maxAttempts) {
        clearInterval(interval);
        setError("Verification timed out - other device didn't respond");
      }
    }, 1000);
  }

  async function confirmMatch() {
    try {
      setStatus("Confirming...");
      await matrixService.confirmVerification();
      setStatus("Verified! Loading keys...");
      
      setTimeout(() => {
        onVerified();
      }, 2000);
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleCancel() {
    try {
      await matrixService.cancelVerification();
      onClose();
    } catch (e) {
      onClose();
    }
  }

  return (
    <div className="verification-overlay">
      <div className="verification-dialog">
        <h2>🔐 Device Verification</h2>

        {step === "request" && (
          <>
            <p>To decrypt encrypted messages, you need to verify this device with another device where you're already logged in (Element).</p>
            <div style={{ 
              background: "#40444b", 
              padding: "15px", 
              borderRadius: "4px",
              marginBottom: "15px",
              fontSize: "14px"
            }}>
              <strong>📱 Before starting:</strong>
              <ol style={{ marginLeft: "20px", marginTop: "10px" }}>
                <li>Open Element on your phone or computer</li>
                <li>Make sure you're logged in with the same account</li>
                <li>Keep Element open and visible</li>
              </ol>
            </div>
            <div className="button-group">
              <button onClick={startVerification} className="primary-btn">
                Start Verification
              </button>
              <button onClick={startVerificationAlternate} className="secondary-btn">
                Use Recovery Key Instead
              </button>
              <button onClick={onClose} className="secondary-btn">
                Skip for Now
              </button>
            </div>
          </>
        )}

        {step === "recovery" && (
          <>
            <p><strong>Enter your Security Key/Recovery Key</strong></p>
            <div style={{ 
              background: "#40444b", 
              padding: "15px", 
              borderRadius: "4px",
              margin: "15px 0",
              fontSize: "14px"
            }}>
              <p>This is the recovery key you saved when you first set up encryption in Element.</p>
              <p style={{ marginTop: "10px" }}>It looks like: <code>EsTc 1234 5678...</code></p>
            </div>
            <input
              type="text"
              value={recoveryKey}
              onChange={(e) => setRecoveryKey(e.target.value)}
              placeholder="Enter recovery key"
              style={{
                width: "100%",
                padding: "10px",
                marginBottom: "15px",
                background: "#40444b",
                border: "1px solid #202225",
                borderRadius: "4px",
                color: "#dcddde",
                fontSize: "14px"
              }}
              autoFocus
            />
            <div className="button-group">
              <button onClick={submitRecoveryKey} className="primary-btn">
                Verify with Key
              </button>
              <button onClick={() => setStep("request")} className="secondary-btn">
                Back
              </button>
            </div>
          </>
        )}

        {step === "waiting" && (
          <>
            <p><strong>{status}</strong></p>
            <div style={{ 
              background: "#40444b", 
              padding: "15px", 
              borderRadius: "4px",
              margin: "15px 0",
              fontSize: "14px"
            }}>
              <strong>On your Element device:</strong>
              <ol style={{ marginLeft: "20px", marginTop: "10px" }}>
                <li>Look for a verification notification (bell icon)</li>
                <li>Tap/click on it</li>
                <li>Choose "Verify with Emoji"</li>
              </ol>
            </div>
            <div className="spinner"></div>
            <p style={{ fontSize: "12px", color: "#8e9297", marginTop: "10px" }}>
              Waiting for you to accept on Element... (up to 60 seconds)
            </p>
            <button onClick={handleCancel} className="secondary-btn">
              Cancel
            </button>
          </>
        )}

        {step === "emoji" && (
          <>
            <p><strong>Compare these emoji with your other device:</strong></p>
            <div className="emoji-grid">
              {emoji.map(([symbol, name], idx) => (
                <div key={idx} className="emoji-item">
                  <div className="emoji-symbol">{symbol}</div>
                  <div className="emoji-name">{name}</div>
                </div>
              ))}
            </div>
            <p>Do the emoji match on both devices?</p>
            <div className="button-group">
              <button onClick={confirmMatch} className="success-btn">
                ✓ They Match
              </button>
              <button onClick={handleCancel} className="danger-btn">
                ✗ They Don't Match
              </button>
            </div>
          </>
        )}

        {error && <p className="error">{error}</p>}
        {status && <p className="status">{status}</p>}
      </div>
    </div>
  );
}
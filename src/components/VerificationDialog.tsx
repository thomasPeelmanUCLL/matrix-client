import { useState, useEffect } from "react";
import { matrixService } from "../services/matrixService";

interface VerificationDialogProps {
  onClose: () => void;
  onVerified: () => void;
}

export function VerificationDialog({ onClose, onVerified }: VerificationDialogProps) {
  const [step, setStep] = useState<"request" | "emoji" | "waiting">("request");
  const [emoji, setEmoji] = useState<[string, string][]>([]);
  const [error, setError] = useState("");
  const [status, setStatus] = useState("");

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

  async function pollForEmoji() {
  const maxAttempts = 60; // Increase to 60 seconds
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
      // Keep polling - errors are expected while waiting
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
      
      // Wait a bit for keys to sync
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
            <button onClick={onClose} className="secondary-btn">
                Skip for Now
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
            Waiting for you to accept on Element... (up to 30 seconds)
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

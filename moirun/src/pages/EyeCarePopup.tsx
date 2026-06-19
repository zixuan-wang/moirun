import { useState, useEffect, useCallback } from "react";
import {
  getSettings,
  confirmEyeCare,
  snoozeEyeCare,
  closeCurrentWindow,
  type EyeCareIntensity,
} from "../services/reminder";

function EyeCarePopup() {
  const [remaining, setRemaining] = useState(20);
  const [intensity, setIntensity] = useState<EyeCareIntensity>("gentle");
  const [completed, setCompleted] = useState(false);
  const [eyeCareInterval, setEyeCareInterval] = useState(30);

  useEffect(() => {
    getSettings()
      .then((s) => {
        const sec = s.eye_care_lock_seconds || 20;
        setRemaining(sec);
        setIntensity(s.eye_care_intensity);
        setEyeCareInterval(s.eye_care_interval_minutes || 30);
      })
      .catch(console.error);
  }, []);

  useEffect(() => {
    if (completed) return;
    if (remaining <= 0) {
      setCompleted(true);
      return;
    }
    const timer = setInterval(() => {
      setRemaining((r) => {
        if (r <= 1) {
          clearInterval(timer);
          setCompleted(true);
          return 0;
        }
        return r - 1;
      });
    }, 1000);
    return () => clearInterval(timer);
  }, [remaining, completed]);

  // 倒计时结束后自动关闭弹窗
  useEffect(() => {
    if (completed) {
      const timeout = setTimeout(() => {
        confirmEyeCare().catch(console.error);
      }, 1500);
      return () => clearTimeout(timeout);
    }
  }, [completed]);

  const handleComplete = useCallback(() => {
    confirmEyeCare().catch(console.error);
  }, []);

  const handleSnooze = useCallback((minutes: number) => {
    snoozeEyeCare(minutes).catch(console.error);
  }, []);

  const canClose = intensity === "gentle" || completed;

  const formatTime = (s: number) => {
    const m = Math.floor(s / 60);
    const sec = s % 60;
    return `${m.toString().padStart(2, "0")}:${sec.toString().padStart(2, "0")}`;
  };

  return (
    <div
      style={{
        width: "100vw",
        height: "100vh",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        background: "#fff",
        fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
        userSelect: "none",
        position: "relative",
      }}
      data-tauri-drag-region
    >
      {canClose && (
        <button
          onClick={() => closeCurrentWindow()}
          style={{
            position: "absolute",
            top: "12px",
            right: "12px",
            width: "28px",
            height: "28px",
            borderRadius: "50%",
            border: "none",
            background: "#eee",
            cursor: "pointer",
            fontSize: "16px",
            lineHeight: 1,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
          }}
        >
          ×
        </button>
      )}

      <div style={{ fontSize: "48px", fontWeight: 700, color: "#4a90d9", marginBottom: "8px" }}>
        {formatTime(remaining)}
      </div>

      <div
        style={{
          fontSize: "16px",
          color: "#666",
          marginBottom: "24px",
          textAlign: "center",
          padding: "0 24px",
          lineHeight: 1.6,
        }}
      >
        {completed
          ? "休息完成，放松一下吧"
          : `您已经持续用眼 ${eyeCareInterval} 分钟，请休息，看远方 1 分钟`}
      </div>

      {intensity !== "gentle" && !completed && (
        <div style={{ fontSize: "12px", color: "#999", marginBottom: "16px" }}>
          倒计时结束后自动关闭
        </div>
      )}

      <div style={{ display: "flex", gap: "12px", flexWrap: "wrap", justifyContent: "center" }}>
        {completed ? (
          <button
            onClick={handleComplete}
            style={{
              padding: "10px 32px",
              borderRadius: "8px",
              border: "none",
              background: "#4a90d9",
              color: "#fff",
              fontSize: "15px",
              cursor: "pointer",
            }}
          >
            已完成休息
          </button>
        ) : (
          <>
            <button
              onClick={handleComplete}
              style={{
                padding: "8px 20px",
                borderRadius: "8px",
                border: "none",
                background: "#4a90d9",
                color: "#fff",
                fontSize: "14px",
                cursor: "pointer",
              }}
            >
              立即完成
            </button>
            <button
              onClick={() => handleSnooze(5)}
              style={{
                padding: "8px 20px",
                borderRadius: "8px",
                border: "1px solid #ddd",
                background: "#fff",
                fontSize: "14px",
                cursor: "pointer",
                color: "#666",
              }}
            >
              延后 5 分钟
            </button>
            <button
              onClick={() => handleSnooze(10)}
              style={{
                padding: "8px 20px",
                borderRadius: "8px",
                border: "1px solid #ddd",
                background: "#fff",
                fontSize: "14px",
                cursor: "pointer",
                color: "#666",
              }}
            >
              延后 10 分钟
            </button>
          </>
        )}
      </div>
    </div>
  );
}

export default EyeCarePopup;

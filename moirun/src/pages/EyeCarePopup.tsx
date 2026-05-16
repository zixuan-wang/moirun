import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

function EyeCarePopup() {
  const [remaining, setRemaining] = useState(20);
  const [intensity, setIntensity] = useState<"gentle" | "locked" | "strict">("gentle");
  const [completed, setCompleted] = useState(false);

  useEffect(() => {
    invoke<{
      eye_care_lock_seconds: number;
      eye_care_intensity: string;
    }>("get_settings")
      .then((s) => {
        const sec = s.eye_care_lock_seconds || 20;
        setRemaining(sec);
        setIntensity(s.eye_care_intensity as typeof intensity);
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

  const handleComplete = useCallback(() => {
    invoke("confirm_eye_care").catch(console.error);
  }, []);

  const handleSnooze = useCallback((minutes: number) => {
    invoke("snooze_eye_care", { minutes }).catch(console.error);
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
          onClick={() => getCurrentWindow().close()}
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

      <div style={{ fontSize: "16px", color: "#666", marginBottom: "24px" }}>
        {completed ? "休息完成，放松一下吧" : "该让眼睛休息一下了"}
      </div>

      {intensity !== "gentle" && !completed && (
        <div style={{ fontSize: "12px", color: "#999", marginBottom: "16px" }}>
          倒计时结束后方可关闭
        </div>
      )}

      <div style={{ display: "flex", gap: "12px" }}>
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

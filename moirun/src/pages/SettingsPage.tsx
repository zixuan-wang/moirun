import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { enable, disable } from "@tauri-apps/plugin-autostart";
import type { AppSettings, DailyStats } from "../types";

const DEFAULT_SETTINGS: AppSettings = {
  water_reminder_enabled: true,
  water_interval_minutes: 30,
  eye_care_enabled: true,
  eye_care_interval_minutes: 30,
  eye_care_intensity: "gentle",
  eye_care_lock_seconds: 20,
  auto_start: false,
};

function SettingsPage() {
  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS);
  const [stats, setStats] = useState<DailyStats>({
    date: "",
    water_count: 0,
    eye_care_count: 0,
  });
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    invoke<AppSettings>("get_settings")
      .then((s) => setSettings(s))
      .catch(console.error);

    invoke<DailyStats>("get_today_stats")
      .then((s) => setStats(s))
      .catch(console.error);

    const unlistenStats = listen<DailyStats>("stats-updated", (e) => {
      setStats(e.payload);
    });

    return () => {
      unlistenStats.then((fn) => fn());
    };
  }, []);

  const updateSetting = <K extends keyof AppSettings>(
    key: K,
    value: AppSettings[K],
  ) => {
    const next = { ...settings, [key]: value };
    setSettings(next);
    invoke("set_settings", { settings: next })
      .then(() => {
        setSaved(true);
        setTimeout(() => setSaved(false), 1000);
      })
      .catch(console.error);

    if (key === "auto_start") {
      if (value === true) {
        enable().catch(console.error);
      } else {
        disable().catch(console.error);
      }
    }
  };

  return (
    <div
      style={{
        padding: "24px",
        fontFamily:
          '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
        color: "#333",
        maxWidth: "480px",
        margin: "0 auto",
      }}
    >
      <h1
        style={{
          fontSize: "22px",
          fontWeight: 600,
          marginBottom: "20px",
          textAlign: "center",
        }}
      >
        眸润设置
      </h1>

      {/* 今日统计 */}
      <div
        style={{
          background: "#f5f7fa",
          borderRadius: "12px",
          padding: "16px",
          marginBottom: "20px",
        }}
      >
        <div style={{ display: "flex", justifyContent: "space-around" }}>
          <div style={{ textAlign: "center" }}>
            <div style={{ fontSize: "28px", fontWeight: 700, color: "#4a90d9" }}>
              {stats.water_count}
            </div>
            <div style={{ fontSize: "12px", color: "#888" }}>今日饮水</div>
          </div>
          <div style={{ textAlign: "center" }}>
            <div style={{ fontSize: "28px", fontWeight: 700, color: "#5cb85c" }}>
              {stats.eye_care_count}
            </div>
            <div style={{ fontSize: "12px", color: "#888" }}>今日护眼</div>
          </div>
        </div>
        <div style={{ display: "flex", gap: "8px", marginTop: "12px" }}>
          <button
            onClick={() =>
              invoke<DailyStats>("confirm_water")
                .then((s) => setStats(s))
                .catch(console.error)
            }
            style={{
              flex: 1,
              padding: "8px",
              borderRadius: "8px",
              border: "none",
              background: "#4a90d9",
              color: "#fff",
              fontSize: "13px",
              cursor: "pointer",
            }}
          >
            已喝水
          </button>
          <button
            onClick={() =>
              invoke<DailyStats>("get_today_stats")
                .then((s) => setStats(s))
                .catch(console.error)
            }
            style={{
              flex: 1,
              padding: "8px",
              borderRadius: "8px",
              border: "1px solid #ddd",
              background: "#fff",
              fontSize: "13px",
              cursor: "pointer",
              color: "#666",
            }}
          >
            刷新统计
          </button>
        </div>
      </div>

      {/* 喝水提醒 */}
      <Section title="喝水提醒">
        <ToggleRow
          label="启用喝水提醒"
          value={settings.water_reminder_enabled}
          onChange={(v) => updateSetting("water_reminder_enabled", v)}
        />
        {settings.water_reminder_enabled && (
          <NumberRow
            label="间隔（分钟）"
            value={settings.water_interval_minutes}
            min={5}
            max={120}
            onChange={(v) => updateSetting("water_interval_minutes", v)}
          />
        )}
      </Section>

      {/* 护眼提醒 */}
      <Section title="护眼提醒">
        <ToggleRow
          label="启用护眼提醒"
          value={settings.eye_care_enabled}
          onChange={(v) => updateSetting("eye_care_enabled", v)}
        />
        {settings.eye_care_enabled && (
          <>
            <NumberRow
              label="间隔（分钟）"
              value={settings.eye_care_interval_minutes}
              min={5}
              max={120}
              onChange={(v) => updateSetting("eye_care_interval_minutes", v)}
            />
            <SelectRow
              label="提醒强度"
              value={settings.eye_care_intensity}
              options={[
                { value: "gentle", label: "温和模式 — 弹窗可随时关闭" },
                { value: "locked", label: "锁时模式 — 倒计时结束后可关闭" },
                { value: "strict", label: "严格模式 — 全屏遮罩覆盖所有显示器" },
              ]}
              onChange={(v) =>
                updateSetting("eye_care_intensity", v as AppSettings["eye_care_intensity"])
              }
            />
            <NumberRow
              label="倒计时锁（秒）"
              value={settings.eye_care_lock_seconds}
              min={10}
              max={60}
              onChange={(v) => updateSetting("eye_care_lock_seconds", v)}
            />
          </>
        )}
      </Section>

      {/* 通用设置 */}
      <Section title="通用">
        <ToggleRow
          label="开机自启"
          value={settings.auto_start}
          onChange={(v) => updateSetting("auto_start", v)}
        />
      </Section>

      {saved && (
        <div
          style={{
            textAlign: "center",
            color: "#5cb85c",
            fontSize: "13px",
            marginTop: "12px",
          }}
        >
          已保存
        </div>
      )}
    </div>
  );
}

function Section({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div
      style={{
        background: "#fff",
        border: "1px solid #e8e8e8",
        borderRadius: "12px",
        padding: "16px",
        marginBottom: "16px",
      }}
    >
      <h2
        style={{
          fontSize: "14px",
          fontWeight: 600,
          marginBottom: "12px",
          color: "#666",
        }}
      >
        {title}
      </h2>
      {children}
    </div>
  );
}

function ToggleRow({
  label,
  value,
  onChange,
}: {
  label: string;
  value: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <div
      style={{
        display: "flex",
        justifyContent: "space-between",
        alignItems: "center",
        padding: "8px 0",
      }}
    >
      <span style={{ fontSize: "14px" }}>{label}</span>
      <button
        onClick={() => onChange(!value)}
        style={{
          width: "44px",
          height: "24px",
          borderRadius: "12px",
          border: "none",
          background: value ? "#4a90d9" : "#ccc",
          cursor: "pointer",
          position: "relative",
          transition: "background 0.2s",
        }}
      >
        <span
          style={{
            position: "absolute",
            top: "2px",
            left: value ? "22px" : "2px",
            width: "20px",
            height: "20px",
            borderRadius: "50%",
            background: "#fff",
            transition: "left 0.2s",
          }}
        />
      </button>
    </div>
  );
}

function NumberRow({
  label,
  value,
  min,
  max,
  onChange,
}: {
  label: string;
  value: number;
  min: number;
  max: number;
  onChange: (v: number) => void;
}) {
  return (
    <div
      style={{
        display: "flex",
        justifyContent: "space-between",
        alignItems: "center",
        padding: "8px 0",
      }}
    >
      <span style={{ fontSize: "14px" }}>{label}</span>
      <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
        <button
          onClick={() => onChange(Math.max(min, value - 1))}
          style={{
            width: "28px",
            height: "28px",
            borderRadius: "6px",
            border: "1px solid #ddd",
            background: "#fff",
            cursor: "pointer",
            fontSize: "16px",
          }}
        >
          -
        </button>
        <span style={{ fontSize: "14px", minWidth: "28px", textAlign: "center" }}>
          {value}
        </span>
        <button
          onClick={() => onChange(Math.min(max, value + 1))}
          style={{
            width: "28px",
            height: "28px",
            borderRadius: "6px",
            border: "1px solid #ddd",
            background: "#fff",
            cursor: "pointer",
            fontSize: "16px",
          }}
        >
          +
        </button>
      </div>
    </div>
  );
}

function SelectRow({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value: string;
  options: { value: string; label: string }[];
  onChange: (v: string) => void;
}) {
  return (
    <div style={{ padding: "8px 0" }}>
      <div style={{ fontSize: "14px", marginBottom: "8px" }}>{label}</div>
      {options.map((opt) => (
        <div
          key={opt.value}
          onClick={() => onChange(opt.value)}
          style={{
            padding: "10px 12px",
            borderRadius: "8px",
            border: value === opt.value ? "2px solid #4a90d9" : "2px solid transparent",
            background: value === opt.value ? "#eef4fc" : "#f9f9f9",
            marginBottom: "6px",
            cursor: "pointer",
            fontSize: "13px",
            transition: "all 0.15s",
          }}
        >
          {opt.label}
        </div>
      ))}
    </div>
  );
}

export default SettingsPage;

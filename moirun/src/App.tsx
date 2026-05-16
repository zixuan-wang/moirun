import { useEffect } from "react";
import { HashRouter, Routes, Route } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";
import { invoke } from "@tauri-apps/api/core";
import SettingsPage from "./pages/SettingsPage";
import EyeCarePopup from "./pages/EyeCarePopup";
import OverlayPage from "./pages/OverlayPage";

function App() {
  useEffect(() => {
    let unlistenWater: (() => void) | null = null;
    let unlistenEye: (() => void) | null = null;
    let unlistenStats: (() => void) | null = null;

    async function setupListeners() {
      // 请求通知权限
      let permissionGranted = await isPermissionGranted();
      if (!permissionGranted) {
        const permission = await requestPermission();
        permissionGranted = permission === "granted";
      }

      // 监听喝水提醒
      const uw = await listen("water-reminder", async () => {
        if (permissionGranted) {
          sendNotification({
            title: "眸润",
            body: "该喝水了，记得站起来活动一下",
          });
        }
      });
      unlistenWater = uw;

      // 监听护眼提醒
      const ue = await listen<string>("eye-care-reminder", async (e) => {
        const intensity = e.payload || "gentle";
        await invoke("open_eye_care_window", { intensity });
      });
      unlistenEye = ue;

      // 监听显示统计
      const us = await listen("show-stats", async () => {
        await invoke("open_settings_window");
      });
      unlistenStats = us;
    }

    setupListeners();

    return () => {
      if (unlistenWater) unlistenWater();
      if (unlistenEye) unlistenEye();
      if (unlistenStats) unlistenStats();
    };
  }, []);

  return (
    <HashRouter>
      <Routes>
        <Route path="/" element={<SettingsPage />} />
        <Route path="/settings" element={<SettingsPage />} />
        <Route path="/eyecare" element={<EyeCarePopup />} />
        <Route path="/overlay" element={<OverlayPage />} />
      </Routes>
    </HashRouter>
  );
}

export default App;

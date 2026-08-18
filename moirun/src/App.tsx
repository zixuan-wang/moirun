import { useEffect } from "react";
import { HashRouter, Routes, Route } from "react-router-dom";
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  onWaterReminder,
  onEyeCareReminder,
  onShowStats,
  openEyeCareWindow,
  openSettingsWindow,
  type EyeCareIntensity,
} from "./services/reminder";
import SettingsPage from "./pages/SettingsPage";
import EyeCarePopup from "./pages/EyeCarePopup";
import OverlayPage from "./pages/OverlayPage";

function App() {
  useEffect(() => {
    let unlistenWater: (() => void) | null = null;
    let unlistenEye: (() => void) | null = null;
    let unlistenStats: (() => void) | null = null;

    async function setupListeners() {
      let permissionGranted = await isPermissionGranted();
      if (!permissionGranted) {
        const permission = await requestPermission();
        permissionGranted = permission === "granted";
      }

      const uw = await onWaterReminder(() => {
        if (permissionGranted) {
          sendNotification({
            title: "眸润",
            body: "该喝水了，记得站起来活动一下",
          });
        }
      });
      unlistenWater = uw;

      const ue = await onEyeCareReminder(async (intensity: EyeCareIntensity) => {
        await openEyeCareWindow(intensity);
      });
      unlistenEye = ue;

      const us = await onShowStats(async () => {
        await openSettingsWindow();
      });
      unlistenStats = us;
    }

    // 提醒事件监听只在 main 窗口注册一次；
    // settings/eyecare/overlay 窗口也加载本组件，若重复注册会导致
    // 重复通知与并发开窗冲突
    if (getCurrentWindow().label === "main") {
      setupListeners();
    }

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

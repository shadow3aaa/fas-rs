"use client";

import type {
  PowerModes as PowerModesType,
  PowerSettings,
} from "@/types/config";
import { PowerModeSettings } from "./PowerModeSettings";

interface PowerModesProps {
  powerModes: PowerModesType;
  updatePowerMode: (
    mode: keyof PowerModesType,
    setting: keyof PowerSettings,
    value: number | number[] | "disabled",
  ) => void;
}

export function PowerModes({ powerModes, updatePowerMode }: PowerModesProps) {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
      {Object.entries(powerModes).map(([mode, settings], index) => (
        <PowerModeSettings
          key={mode}
          mode={mode as keyof PowerModesType}
          settings={settings}
          updatePowerMode={updatePowerMode}
          index={index}
        />
      ))}
    </div>
  );
}

"use client";

import type { PowerModes, PowerSettings } from "@/types/config";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Slider } from "@/components/ui/slider";
import { Switch } from "@/components/ui/switch";
import { useTranslation } from "react-i18next";
import { Zap, Thermometer } from "lucide-react";

interface PowerModeSettingsProps {
  mode: keyof PowerModes;
  settings: PowerSettings;
  updatePowerMode: (
    mode: keyof PowerModes,
    setting: keyof PowerSettings,
    value: number | number[] | "disabled",
  ) => void;
  index: number;
}

export function PowerModeSettings({
  mode,
  settings,
  updatePowerMode,
  index, // eslint-disable-line @typescript-eslint/no-unused-vars
}: PowerModeSettingsProps) {
  const { t } = useTranslation();

  // Get appropriate icon color based on power mode
  const getModeColor = () => {
    switch (mode) {
      case "powersave":
        return "text-green-500";
      case "balance":
        return "text-blue-500";
      case "performance":
        return "text-orange-500";
      case "fast":
        return "text-red-500";
      default:
        return "text-primary";
    }
  };

  return (
    <Card
      key={mode}
      className="border border-border/40 shadow-sm overflow-hidden"
    >
      <CardHeader className="pb-2 border-b border-border/20 flex flex-row items-center gap-3">
        <Zap className={`h-5 w-5 ${getModeColor()}`} />

        <div>
          <CardTitle className="text-lg capitalize font-bold">
            {t(`common:${mode}_mode`)}
          </CardTitle>
          <CardDescription>{t(`common:${mode}_mode_desc`)}</CardDescription>
        </div>
      </CardHeader>
      <CardContent className="p-0">
        <div className="p-4 border-b border-border/10">
          <div className="flex justify-between items-center mb-2">
            <div className="flex items-center gap-2">
              <span className="text-base font-medium">
                {t("common:fas_margin")}
              </span>
            </div>
            <span className="text-sm text-foreground px-2 py-1 bg-muted/30 rounded-full">
              {settings.margin_fps.toFixed(1)} fps
            </span>
          </div>
          <Slider
            value={[settings.margin_fps]}
            min={0}
            max={5}
            step={0.1}
            onValueChange={(value) =>
              updatePowerMode(mode, "margin_fps", value)
            }
            className="py-3 [&_[data-part=thumb]]:h-6 [&_[data-part=thumb]]:w-6 [&_[data-part=track]]:h-3 [&_[data-part=thumb]]:bg-primary [&_[data-part=track]]:bg-muted"
          />
          <p className="text-sm text-muted-foreground mt-3">
            {t("common:margin")}
          </p>
        </div>

        <div className="p-4">
          <div className="flex items-center justify-between mb-2">
            <div className="flex items-center gap-2">
              <Thermometer className="h-4 w-4 text-red-500" />
              <span className="text-base font-medium">
                {t("common:thermal")}
              </span>
              <Switch
                checked={settings.core_temp_thresh !== "disabled"}
                onCheckedChange={(checked: boolean) =>
                  updatePowerMode(
                    mode,
                    "core_temp_thresh",
                    checked
                      ? Number(
                          localStorage.getItem(`fasrs-${mode}-temp`) || 80000,
                        )
                      : "disabled",
                  )
                }
                className="data-[state=checked]:bg-primary"
              />
            </div>
            <span className="text-sm text-foreground px-2 py-1 bg-muted/30 rounded-full">
              {settings.core_temp_thresh === "disabled"
                ? t("common:disabled")
                : `${Math.round(settings.core_temp_thresh / 1000)}°C`}
            </span>
          </div>
          <Slider
            value={
              settings.core_temp_thresh === "disabled"
                ? [80000]
                : [settings.core_temp_thresh as number]
            }
            min={60000}
            max={100000}
            step={1000}
            disabled={settings.core_temp_thresh === "disabled"}
            onValueChange={(value) => {
              localStorage.setItem(`fasrs-${mode}-temp`, value[0].toString());
              updatePowerMode(mode, "core_temp_thresh", value[0]);
            }}
            className="py-3 [&_[data-part=thumb]]:h-6 [&_[data-part=thumb]]:w-6 [&_[data-part=track]]:h-3 data-[disabled]:opacity-50 [&_[data-part=thumb]]:bg-primary [&_[data-part=track]]:bg-muted"
          />
          <p className="text-sm text-muted-foreground mt-3">
            {t("common:thermal_desc")}
          </p>
        </div>
      </CardContent>
    </Card>
  );
}

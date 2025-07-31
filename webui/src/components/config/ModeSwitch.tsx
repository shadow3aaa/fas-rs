"use client";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";
import {
  Zap,
  BatteryCharging,
  SlidersHorizontal,
  Gauge,
} from "lucide-react";
import { exec } from "@/lib/kernelsu";
import { useEffect, useState } from "react";

const MODES = ["powersave", "balance", "performance", "fast"] as const;

const MODE_ICONS = {
  powersave: BatteryCharging,
  balance: SlidersHorizontal,
  performance: Gauge,
  fast: Zap,
} as const;

type Mode = (typeof MODES)[number];

export function ModeSwitch() {
  const { t } = useTranslation();
  const [currentMode, setCurrentMode] = useState<Mode | null>(null);

  useEffect(() => {
    const fetchMode = async () => {
    //   if (process.env.NODE_ENV === "development") {
    //     setCurrentMode("balance");
    //     return;
    //   }

      try {
        const { errno, stdout } = await exec(`cat /dev/fas_rs/mode`, { cwd: "/" });
        if (errno === 0) {
          const mode = stdout.trim() as Mode;
          if (MODES.includes(mode)) {
            setCurrentMode(mode);
          }
        }
      } catch (_e) {
        /* ignore */
      }
    };

    fetchMode();
  }, []);

  const getModeColor = (mode: Mode) => {
    switch (mode) {
      case "powersave":
        return "bg-green-500 hover:bg-green-600 text-white";
      case "balance":
        return "bg-blue-500 hover:bg-blue-600 text-white";
      case "performance":
        return "bg-orange-500 hover:bg-orange-600 text-white";
      case "fast":
        return "bg-red-500 hover:bg-red-600 text-white";
      default:
        return "";
    }
  };

  const switchMode = async (mode: Mode) => {
    // if (process.env.NODE_ENV === "development") {
    //   setCurrentMode(mode);
    //   toast.info(`Switch mode to ${mode} (skipped in dev)`);
    //   console.log(`Switch mode to ${mode} (skipped in dev)`);
    //   return;
    // }

    try {
      const { errno, stderr } = await exec(`echo -n ${mode} > /dev/fas_rs/mode`, {
        cwd: "/",
      });

      if (errno === 0) {
        setCurrentMode(mode);
        toast.success(`Switched to ${t(`common:${mode}_mode`)}`);
      } else {
        toast.error(`Failed to switch mode: ${stderr}`);
      }
    } catch (error) {
      toast.error(`Failed to switch mode: ${error}`);
    }
  };

  return (
    <Card className="overflow-hidden shadow-sm border border-border/40">
      <CardHeader className="pb-2 border-b border-border/20">
        <div>
          <CardTitle className="text-lg font-bold capitalize">
            {t("common:tab_power")}
          </CardTitle>
          {currentMode && (
            <CardDescription className="capitalize">
              {t(`common:${currentMode}_mode`)}
            </CardDescription>
          )}
        </div>
      </CardHeader>
      <CardContent className="p-4">
        <div className="grid grid-cols-2 gap-4">
          {MODES.map((mode) => {
            const Icon = MODE_ICONS[mode];
            return (
              <Button
                key={mode}
                onClick={() => switchMode(mode)}
                className={`capitalize flex items-center gap-2 ${mode === currentMode ? getModeColor(mode) : ""}`}
                variant={mode === currentMode ? "default" : "outline"}
              >
                <Icon className="h-4 w-4" />
                {t(`common:${mode}_mode`)}
              </Button>
            );
          })}
        </div>
      </CardContent>
    </Card>
  );
} 
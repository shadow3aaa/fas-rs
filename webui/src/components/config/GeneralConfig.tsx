"use client";

import type { ConfigOptions } from "@/types/config";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { useTranslation } from "react-i18next";

interface GeneralConfigProps {
  configOptions: ConfigOptions;
  toggleConfigOption: (option: keyof ConfigOptions) => void;
}

export function GeneralConfig({
  configOptions,
  toggleConfigOption,
}: GeneralConfigProps) {
  const { t } = useTranslation();

  return (
    <Card className="overflow-hidden shadow-sm border border-border/40">
      <CardHeader className="pb-2 border-b border-border/20">
        <CardTitle className="text-lg font-bold">
          {t("common:config")}
        </CardTitle>
        <CardDescription>{t("common:basic_settings")}</CardDescription>
      </CardHeader>
      <CardContent className="p-0">
        <div className="p-4 border-b border-border/10 hover:bg-muted/5">
          <div className="flex items-center justify-between">
            <div className="space-y-1">
              <label className="text-base font-medium">
                {t("common:keep_std")}
              </label>
              <CardDescription className="text-xs">
                {t("common:keep_std_desc")}
              </CardDescription>
            </div>
            <Switch
              checked={configOptions.keep_std}
              onCheckedChange={() => toggleConfigOption("keep_std")}
              className="data-[state=checked]:bg-primary"
            />
          </div>
        </div>

        <div className="p-4 hover:bg-muted/5">
          <div className="flex items-center justify-between">
            <div className="space-y-1">
              <label className="text-base font-medium">
                {t("common:scene_game_list")}
              </label>
              <CardDescription className="text-xs">
                {t("common:scene_game_list_desc")}
              </CardDescription>
            </div>
            <Switch
              checked={configOptions.scene_game_list}
              onCheckedChange={() => toggleConfigOption("scene_game_list")}
              className="data-[state=checked]:bg-primary"
            />
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

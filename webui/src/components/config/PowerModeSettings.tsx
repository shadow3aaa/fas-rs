import { PowerModes, PowerSettings } from "@/types/config";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Slider } from "@/components/ui/slider";
import { Switch } from "@/components/ui/switch";
import { Separator } from "@/components/ui/separator";
import { useTranslation } from 'react-i18next';

interface PowerModeSettingsProps {
  mode: keyof PowerModes;
  settings: PowerSettings;
  updatePowerMode: (mode: keyof PowerModes, setting: keyof PowerSettings, value: number | number[] | "disabled") => void;
}

export function PowerModeSettings({ mode, settings, updatePowerMode }: PowerModeSettingsProps) {
  const { t } = useTranslation();
  return (
    <Card key={mode}>
      <CardHeader>
        <CardTitle className="text-lg capitalize">{t(`common:${mode}_mode`)}</CardTitle>
        <CardDescription>{t(`common:${mode}_mode_desc`)}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="space-y-4">
          <div className="flex justify-between items-center mb-2">
            <span className="text-base font-medium">{t('common:fas_margin')}</span>
            <span className="text-sm text-gray-600">{settings.margin_fps.toFixed(1)} fps</span>
          </div>
          <Slider
            value={[settings.margin_fps]}
            min={0}
            max={5}
            step={0.1}
            onValueChange={(value) => updatePowerMode(mode, "margin_fps", value)}
            className="py-3 [&_[data-part=thumb]]:h-6 [&_[data-part=thumb]]:w-6 [&_[data-part=track]]:h-3"
          />
          <p className="text-sm text-gray-600 mt-3">
            {t('common:margin')}
          </p>
        </div>

        <Separator className="my-4" />

        <div className="space-y-4">
          <div className="flex items-center justify-between mb-2">
            <div className="flex items-center gap-2">
              <span className="text-base font-medium">{t('common:thermal')}</span>
              <Switch
                checked={settings.core_temp_thresh !== "disabled"}
                onCheckedChange={(checked: boolean) => 
                  updatePowerMode(
                    mode, 
                    "core_temp_thresh", 
                    checked ? Number(localStorage.getItem(`fasrs-${mode}-temp`) || 80000) : "disabled"
                  )
                }
              />
            </div>
            <span className="text-sm text-gray-600 ml-2">
              {settings.core_temp_thresh === "disabled" 
                ? t('common:disabled')
                : `${Math.round(settings.core_temp_thresh / 1000)}°C`}
            </span>
          </div>
          <Slider
                value={settings.core_temp_thresh === "disabled" ? [80000] : [settings.core_temp_thresh as number]}
            min={60000}
            max={100000}
            step={1000}
            disabled={settings.core_temp_thresh === "disabled"}
            onValueChange={(value) => {
              localStorage.setItem(`fasrs-${mode}-temp`, value[0].toString());
              updatePowerMode(mode, "core_temp_thresh", value[0]);
            }}
            className="py-3 [&_[data-part=thumb]]:h-6 [&_[data-part=thumb]]:w-6 [&_[data-part=track]]:h-3 data-[disabled]:opacity-50"
          />
          <p className="text-sm text-gray-600 mt-3">
            {t('common:thermal_desc')}
          </p>
        </div>
      </CardContent>
    </Card>
  );
}

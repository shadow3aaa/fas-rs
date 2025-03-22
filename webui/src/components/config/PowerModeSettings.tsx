import { PowerModes, PowerSettings } from "@/types/config";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Slider } from "@/components/ui/slider";
import { Separator } from "@/components/ui/separator";
import { useTranslation } from 'react-i18next';

interface PowerModeSettingsProps {
  mode: keyof PowerModes;
  settings: PowerSettings;
  updatePowerMode: (mode: keyof PowerModes, setting: keyof PowerSettings, value: number | number[]) => void;
}

export function PowerModeSettings({ mode, settings, updatePowerMode }: PowerModeSettingsProps) {
  const { t } = useTranslation();
  return (
    <Card key={mode}>
      <CardHeader>
        <CardTitle className="text-lg capitalize">{mode}</CardTitle>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="space-y-4">
          <div className="flex justify-between items-center mb-2">
            <span className="text-base font-medium">{t('common:performance_margin')}: {settings.margin_fps.toFixed(1)} fps</span>
            <span className="text-sm text-gray-600">{t(`common:${mode}`)}</span>
          </div>
          <Slider
            value={[settings.margin_fps]}
            min={0}
            max={5}
            step={0.1}
            onValueChange={(value) => updatePowerMode(mode, "margin_fps", value)}
            className="py-3 [&_[data-part=thumb]]:h-6 [&_[data-part=thumb]]:w-6 [&_[data-part=track]]:h-3"
          />
        </div>

        <Separator className="my-4" />

        <div className="space-y-4">
          <div className="flex justify-between items-center mb-2">
            <span className="text-base font-medium">{t('common:temperature_limit')}: {Math.round(settings.core_temp_thresh / 1000)}°C</span>
            <span className="text-sm text-gray-600">{t('common:threshold')}</span>
          </div>
          <Slider
            value={[settings.core_temp_thresh]}
            min={60000}
            max={100000}
            step={5000}
            onValueChange={(value) => updatePowerMode(mode, "core_temp_thresh", value)}
            className="py-3 [&_[data-part=thumb]]:h-6 [&_[data-part=thumb]]:w-6 [&_[data-part=track]]:h-3"
          />
          <p className="text-sm text-gray-600 mt-3">
            {t('common:thermal_throttling')}
          </p>
        </div>
      </CardContent>
    </Card>
  );
}

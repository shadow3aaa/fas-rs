import { ConfigOptions } from "@/types/config";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { useTranslation } from 'react-i18next';

interface GeneralConfigProps {
  configOptions: ConfigOptions;
  toggleConfigOption: (option: keyof ConfigOptions) => void;
}

export function GeneralConfig({ configOptions, toggleConfigOption }: GeneralConfigProps) {
  const { t } = useTranslation();
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-lg">{t('common:config')}</CardTitle>
        <CardDescription>
          {t('common:basic_settings')}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <label className="text-base font-medium">{t('common:keep_std')}</label>
            <CardDescription>
              {t("common:keep_std_desc")}
            </CardDescription>
          </div>
          <Switch
            checked={configOptions.keep_std}
            onCheckedChange={() => toggleConfigOption("keep_std")}
          />
        </div>

        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <label className="text-base font-medium">{t('common:scene_game_list')}</label>
            <CardDescription>
              {t("common:scene_game_list_desc")}
            </CardDescription>
          </div>
          <Switch
            checked={configOptions.scene_game_list}
            onCheckedChange={() => toggleConfigOption("scene_game_list")}
          />
        </div>
      </CardContent>
    </Card>
  );
}

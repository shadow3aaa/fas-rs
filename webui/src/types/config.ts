export type ConfigOptions = {
  keep_std: boolean;
  scene_game_list: boolean;
  language: 'en' | 'zh';
};

export type PowerSettings = {
  margin_fps: number;
  core_temp_thresh: number | "disabled";
};

export type UpdatePowerModeFn = (
  mode: keyof PowerModes, 
  setting: keyof PowerSettings, 
  value: number | number[] | "disabled"
) => void;

export type PowerModes = {
  powersave: PowerSettings;
  balance: PowerSettings;
  performance: PowerSettings;
  fast: PowerSettings;
};

export type FpsValue = number | number[];

export type GameList = {
  [packageName: string]: FpsValue;
};

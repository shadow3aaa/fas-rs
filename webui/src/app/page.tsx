"use client";

import { useTranslation } from "react-i18next";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useConfig } from "@/hooks/useConfig";
import { GeneralConfig } from "@/components/config/GeneralConfig";
import { GameList } from "@/components/config/GameList";
import { PowerModes } from "@/components/config/PowerModes";
import { Settings, Gamepad, Zap } from "lucide-react";

export default function Home() {
  const { t } = useTranslation();
  const {
    configOptions,
    gameList,
    powerModes,
    newGamePackage,
    setNewGamePackage,
    newGameFps,
    setNewGameFps,
    isAddingGame,
    setIsAddingGame,
    editingGame,
    editingGameFps,
    setEditingGameFps,
    toggleConfigOption,
    updatePowerMode,
    addNewGame,
    removeGame,
    startEditGame,
    saveEditedGame,
  } = useConfig();

  return (
    <div className="flex flex-col min-h-screen pt-16">
      {/* Header */}
      <div className="p-4 border-b border-border/20">
        <h1 className="text-xl font-bold text-primary">
          {t("common:games_config")}
        </h1>
        <p className="text-sm text-muted-foreground mt-1">
          {t("common:manage_settings")}
        </p>
      </div>

      <div className="p-4">
        <Tabs defaultValue="config" className="w-full">
          <TabsList className="w-full justify-start rounded-xl mb-4 p-1 bg-muted/50">
            <TabsTrigger
              value="config"
              className="flex items-center gap-2 data-[state=active]:bg-background data-[state=active]:shadow-sm rounded-lg transition-all duration-200"
            >
              <Settings className="h-4 w-4" />
              <span>{t("common:tab_general")}</span>
            </TabsTrigger>
            <TabsTrigger
              value="games"
              className="flex items-center gap-2 data-[state=active]:bg-background data-[state=active]:shadow-sm rounded-lg transition-all duration-200"
            >
              <Gamepad className="h-4 w-4" />
              <span>{t("common:tab_games")}</span>
            </TabsTrigger>
            <TabsTrigger
              value="power"
              className="flex items-center gap-2 data-[state=active]:bg-background data-[state=active]:shadow-sm rounded-lg transition-all duration-200"
            >
              <Zap className="h-4 w-4" />
              <span>{t("common:tab_power")}</span>
            </TabsTrigger>
          </TabsList>

          {/* General Config Tab */}
          <TabsContent value="config" className="space-y-4">
            <GeneralConfig
              configOptions={configOptions}
              toggleConfigOption={toggleConfigOption}
            />
          </TabsContent>

          {/* Game List Tab */}
          <TabsContent value="games" className="space-y-4">
            <GameList
              gameList={gameList}
              newGamePackage={newGamePackage}
              setNewGamePackage={setNewGamePackage}
              newGameFps={newGameFps}
              setNewGameFps={setNewGameFps}
              isAddingGame={isAddingGame}
              setIsAddingGame={setIsAddingGame}
              editingGame={editingGame}
              editingGameFps={editingGameFps}
              setEditingGameFps={setEditingGameFps}
              addNewGame={addNewGame}
              removeGame={removeGame}
              startEditGame={startEditGame}
              saveEditedGame={saveEditedGame}
            />
          </TabsContent>

          {/* Power Modes Tab */}
          <TabsContent value="power" className="space-y-4">
            <PowerModes
              powerModes={powerModes}
              updatePowerMode={updatePowerMode}
            />
          </TabsContent>
        </Tabs>
      </div>
    </div>
  );
}

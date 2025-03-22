"use client";

import { Save } from "lucide-react";
import { useTranslation } from 'react-i18next';
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { useConfig } from "@/hooks/useConfig";
import { GeneralConfig } from "@/components/config/GeneralConfig";
import { GameList } from "@/components/config/GameList";
import { PowerModes } from "@/components/config/PowerModes";

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
    saveConfiguration
  } = useConfig();

  return (
    <div className="min-h-screen bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-slate-900 dark:to-slate-800 p-4">
      {/* Main content */}
      <Card className="max-w-4xl mx-auto mb-8 bg-white/80 dark:bg-gray-800/80 backdrop-blur">
        <CardHeader className="border-b border-gray-200 dark:border-gray-700 bg-gray-50/80 dark:bg-gray-900/80">
          <div className="flex justify-between items-center">
            <CardTitle>{t('common:games_config')}</CardTitle>
            <Button onClick={saveConfiguration} className="gap-2">
              <Save className="w-4 h-4" />
              {t('common:save_configuration')}
            </Button>
          </div>
          <CardDescription>
            {t('common:manage_settings')}
          </CardDescription>
        </CardHeader>

        <CardContent className="p-0">
          <Tabs defaultValue="config" className="w-full">
            <TabsList className="w-full justify-start rounded-none border-b px-2 space-x-4">
              <TabsTrigger value="config" className="text-lg px-6 py-3">{t('common:tab_general')}</TabsTrigger>
              <TabsTrigger value="games" className="text-lg px-6 py-3">{t('common:tab_games')}</TabsTrigger>
              <TabsTrigger value="power" className="text-lg px-6 py-3">{t('common:tab_power')}</TabsTrigger>
            </TabsList>
            
            {/* General Config Tab */}
            <TabsContent value="config" className="p-6">
              <GeneralConfig 
                configOptions={configOptions}
                toggleConfigOption={toggleConfigOption}
              />
            </TabsContent>

            {/* Game List Tab */}
            <TabsContent value="games" className="p-6">
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
            <TabsContent value="power" className="p-6">
              <PowerModes 
                powerModes={powerModes}
                updatePowerMode={updatePowerMode}
              />
            </TabsContent>
          </Tabs>
        </CardContent>
      </Card>
    </div>
  );
}

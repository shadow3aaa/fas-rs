"use client";

import { useTranslation } from 'react-i18next';
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
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
  } = useConfig();

  return (
    <div>
      {/* Main content */}
      <div>
        <div className="border-b border-gray-200 dark:border-gray-700 p-6 bg-gray-50/80 dark:bg-gray-900/80">
          <div className="flex justify-between items-center">
            <h1 className="text-2xl font-semibold">{t('common:games_config')}</h1>
          </div>
          <p className="text-sm text-muted-foreground mt-2">
            {t('common:manage_settings')}
          </p>
        </div>

        <div className="p-0">
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
        </div>
      </div>
    </div>
  );
}

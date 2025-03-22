import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { GameItem } from "./GameItem";
import { GameList as GameListType, FpsValue } from "@/types/config";
import { Plus } from "lucide-react";
import { useTranslation } from 'react-i18next';

interface GameListProps {
  gameList: GameListType;
  newGamePackage: string;
  setNewGamePackage: (value: string) => void;
  newGameFps: string;
  setNewGameFps: (value: string) => void;
  isAddingGame: boolean;
  setIsAddingGame: (value: boolean) => void;
  editingGame: string | null;
  editingGameFps: string;
  setEditingGameFps: (value: string) => void;
  addNewGame: () => void;
  removeGame: (game: string) => void;
  startEditGame: (game: string, fps: FpsValue) => void;
  saveEditedGame: () => void;
}

export function GameList({
  gameList,
  newGamePackage,
  setNewGamePackage,
  newGameFps,
  setNewGameFps,
  isAddingGame,
  setIsAddingGame,
  editingGame,
  editingGameFps,
  setEditingGameFps,
  addNewGame,
  removeGame,
  startEditGame,
  saveEditedGame
}: GameListProps) {
  const { t } = useTranslation();
  return (
    <div className="relative">
      <Card>
        <CardHeader className="flex flex-row items-center justify-between pb-2">
          <div>
            <CardTitle className="text-lg">{t('common:game_list')}</CardTitle>
            <CardDescription>
              {t('common:configure_fps')}
            </CardDescription>
          </div>
          {!isAddingGame && (
            <Button onClick={() => setIsAddingGame(true)} variant="outline" size="sm" className="gap-1">
              <Plus className="h-4 w-4" />
              {t('common:add_game')}
            </Button>
          )}
        </CardHeader>

        <CardContent className="space-y-4">
          {/* Add new game form */}
          {isAddingGame && (
            <Card className="bg-gray-50 dark:bg-gray-900 mb-4">
              <CardHeader className="pb-2">
                <CardTitle className="text-md">{t('common:add_game')}</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <label className="text-base font-medium">{t('common:package_name')}</label>
                  <Input
                    type="text"
                    value={newGamePackage}
                    onChange={(e) => setNewGamePackage(e.target.value)}
                    placeholder="com.example.game"
                    className="min-h-[64px] text-xl whitespace-pre-wrap leading-relaxed"
                    style={{ whiteSpace: 'pre-wrap' }}
                    inputMode="text"
                  />
                </div>
                <div className="space-y-2">
                  <label className="text-base font-medium">{t('common:fps_values')}</label>
                  <Input
                    type="text"
                    value={newGameFps}
                    onChange={(e) => setNewGameFps(e.target.value)}
                    placeholder="30, 60, 90"
                    className="h-16 text-xl"
                  />
                </div>
              </CardContent>
              <CardFooter className="flex justify-end gap-2">
                <Button
                  onClick={() => setIsAddingGame(false)}
                  variant="outline"
                  size="lg"
                  className="px-8 py-5 text-lg"
                >
                  {t('common:cancel')}
                </Button>
                <Button
                  onClick={addNewGame}
                  size="lg"
                  className="px-10 py-5 text-lg"
                >
                  {t('common:add')}
                </Button>
              </CardFooter>
            </Card>
          )}

          {/* Game list */}
          <div className="space-y-4 overflow-x-auto">
            <div className="grid grid-cols-1 gap-4">
              {Object.entries(gameList).map(([game, fps]) => (
                <GameItem
                  key={game}
                  game={game}
                  fps={fps}
                  editingGame={editingGame}
                  editingGameFps={editingGameFps}
                  setEditingGameFps={setEditingGameFps}
                  startEditGame={startEditGame}
                  saveEditedGame={saveEditedGame}
                  removeGame={removeGame}
                />
              ))}
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

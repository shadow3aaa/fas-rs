"use client";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { GameItem } from "./GameItem";
import type { GameList as GameListType, FpsValue } from "@/types/config";
import { Plus, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useRef, useState, useEffect } from "react";
import { Combobox } from "@/components/ui/combobox";
import { useQuery } from "@tanstack/react-query";
import { fetchApps } from "@/lib/api";

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
  saveEditedGame,
}: GameListProps) {
  const { t } = useTranslation();
  const [isPopupVisible, setIsPopupVisible] = useState(false);
  const fpsInputRef = useRef<HTMLInputElement>(null);
  const { data: apps = [], isLoading } = useQuery({
    queryKey: ["apps"],
    queryFn: fetchApps,
  });

  useEffect(() => {
    if (isAddingGame) {
      setTimeout(() => {
        setIsPopupVisible(true);
      }, 50);
    } else {
      setIsPopupVisible(false);
    }
  }, [isAddingGame]);

  return (
    <div className="relative">
      <Card className="shadow-sm border border-border/40">
        <CardHeader className="pb-2 border-b border-border/20 flex flex-row items-center justify-between">
          <div>
            <CardTitle className="text-lg font-bold">
              {t("common:game_list")}
            </CardTitle>
            <CardDescription>{t("common:configure_fps")}</CardDescription>
          </div>
          {!isAddingGame && (
            <Button
              onClick={() => setIsAddingGame(true)}
              size="icon"
              className="h-8 w-8 rounded-full bg-green-500 hover:bg-green-600 text-white"
            >
              <Plus className="h-4 w-4" strokeWidth={5} />
            </Button>
          )}
        </CardHeader>

        <CardContent className="p-4 space-y-4">
          {/* Game list */}
          <div className="space-y-4 overflow-x-auto">
            <div className="grid grid-cols-1 gap-4">
              {Object.entries(gameList).length === 0 ? (
                <div className="text-center py-8 text-muted-foreground">
                  {t("common:no_games")}
                </div>
              ) : (
                Object.entries(gameList).map(([game, fps], index) => (
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
                    index={index}
                  />
                ))
              )}
            </div>
          </div>
        </CardContent>
      </Card>

      {isAddingGame && (
        <div
          className={`fixed top-0 left-0 right-0 z-50 mx-auto max-w-md p-4 transition-opacity duration-300 ${
            isPopupVisible ? "opacity-100" : "opacity-0"
          }`}
        >
          <div
            className="fixed inset-0 bg-black/80 backdrop-blur-sm z-40 transition-opacity duration-300"
            style={{ opacity: isPopupVisible ? 1 : 0 }}
            onClick={() => setIsAddingGame(false)}
          />

          <Card className="bg-card border-border shadow-lg mb-4 relative z-50 rounded-xl">
            <Button
              onClick={() => setIsAddingGame(false)}
              variant="ghost"
              size="icon"
              className="absolute right-2 top-2 h-8 w-8 rounded-full bg-muted/50 hover:bg-muted text-foreground border-none"
            >
              <X className="h-4 w-4" />
            </Button>

            <CardHeader className="pb-2 pt-4">
              <CardTitle className="text-xl text-foreground">
                {t("common:add_game")}
              </CardTitle>
            </CardHeader>

            <CardContent className="space-y-4">
              <div className="space-y-1.5">
                <label className="text-base font-medium text-foreground">
                  {t("common:package_name")}
                </label>
                <Combobox
                  value={newGamePackage}
                  onValueChange={setNewGamePackage}
                  options={apps.map((app) => ({
                    label: app.package_name,
                    value: app.package_name,
                    disabled: Object.keys(gameList).includes(app.package_name),
                  }))}
                  placeholder={t("common:search_app")}
                  emptyText={
                    isLoading ? t("common:loading") : t("common:no_apps_found")
                  }
                  searchText={t("common:search_app")}
                  className="bg-card border-border"
                />
              </div>

              <div className="space-y-1.5">
                <label className="text-base font-medium text-foreground">
                  {t("common:fps_values")}{" "}
                  <span className="text-muted-foreground">(comma_separated)</span>
                </label>
                <Input
                  ref={fpsInputRef}
                  type="text"
                  value={newGameFps}
                  onChange={(e) => setNewGameFps(e.target.value)}
                  placeholder="30, 60, 90"
                  className="h-10 bg-card border-border focus-visible:ring-offset-0 focus-visible:ring-primary"
                />
              </div>
            </CardContent>

            <CardFooter className="flex justify-end gap-2 pt-2">
              <Button
                onClick={() => setIsAddingGame(false)}
                variant="outline"
                className="border-border hover:bg-muted hover:text-foreground"
              >
                {t("common:cancel")}
              </Button>
              <Button
                onClick={addNewGame}
                variant="default"
                className="bg-primary text-primary-foreground hover:bg-primary/90"
              >
                {t("common:add")}
              </Button>
            </CardFooter>
          </Card>
        </div>
      )}
    </div>
  );
}

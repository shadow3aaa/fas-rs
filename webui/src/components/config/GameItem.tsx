"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import type { FpsValue } from "@/types/config";
import { Pencil, Save, Trash2, X, Gamepad } from "lucide-react";
import { DeleteGameDialog } from "./DeleteGameDialog";

interface GameItemProps {
  game: string;
  fps: FpsValue;
  editingGame: string | null;
  editingGameFps: string;
  setEditingGameFps: (value: string) => void;
  startEditGame: (game: string, fps: FpsValue) => void;
  saveEditedGame: () => void;
  removeGame: (game: string) => void;
  index: number;
}

export function GameItem({
  game,
  fps,
  editingGame,
  editingGameFps,
  setEditingGameFps,
  startEditGame,
  saveEditedGame,
  removeGame,
  index, // eslint-disable-line @typescript-eslint/no-unused-vars
}: GameItemProps) {
  const isEditing = editingGame === game;
  const [showDeleteDialog, setShowDeleteDialog] = useState(false);

  const handleDelete = () => {
    setShowDeleteDialog(true);
  };

  const confirmDelete = () => {
    removeGame(game);
    setShowDeleteDialog(false);
  };

  return (
    <>
      <Card
        key={game}
        className="border border-border/40 shadow-sm hover:border-border transition-all duration-200"
      >
        <CardContent className="p-0">
          {isEditing ? (
            <div className="p-4 space-y-3">
              <div className="flex items-center gap-2 w-full">
                <Gamepad className="h-5 w-5 text-primary" />
                <span className="font-mono text-sm sm:text-base w-full break-words font-medium">
                  {game}
                </span>
              </div>
              <Input
                type="text"
                value={editingGameFps}
                onChange={(e) => setEditingGameFps(e.target.value)}
                className="w-full text-sm sm:text-base mb-3 focus-visible:ring-primary"
                placeholder="Enter FPS"
              />
              <div className="flex space-x-3 w-full">
                <Button
                  onClick={() => startEditGame("", fps)}
                  variant="destructive"
                  size="sm"
                  className="h-10 w-10 rounded-full"
                >
                  <X className="h-5 w-5" />
                </Button>
                <Button
                  onClick={saveEditedGame}
                  size="sm"
                  className="h-10 w-10 rounded-full"
                >
                  <Save className="h-5 w-5" />
                </Button>
              </div>
            </div>
          ) : (
            <div className="p-4 space-y-3">
              <div className="flex items-center gap-2">
                <Gamepad className="h-5 w-5 text-primary" />
                <span className="font-mono text-sm sm:text-base break-words w-full font-medium">
                  {game}
                </span>
              </div>
              <div className="pl-7">
                <span className="text-muted-foreground text-sm">FPS: </span>
                <span className="text-primary font-medium">
                  {Array.isArray(fps) ? fps.join(", ") : fps}
                </span>
              </div>
              <div className="flex space-x-3 w-full mt-2">
                <Button
                  onClick={() => startEditGame(game, fps)}
                  variant="secondary"
                  size="sm"
                  className="h-9 w-9 rounded-full"
                >
                  <Pencil className="h-4 w-4" />
                </Button>
                <Button
                  onClick={handleDelete}
                  variant="destructive"
                  size="sm"
                  className="h-9 w-9 rounded-full"
                >
                  <Trash2 className="h-4 w-4" />
                </Button>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      <DeleteGameDialog
        isOpen={showDeleteDialog}
        onClose={() => setShowDeleteDialog(false)}
        onConfirm={confirmDelete}
        gameName={game}
      />
    </>
  );
}

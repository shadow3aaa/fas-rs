"use client";

import { useState, useRef, useEffect } from "react";
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
  const [isPopupVisible, setIsPopupVisible] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (isEditing) {
      setTimeout(() => {
        setIsPopupVisible(true);
      }, 50);

      setTimeout(() => {
        if (inputRef.current) {
          inputRef.current.focus();
        }
      }, 350);
    } else {
      setIsPopupVisible(false);
    }
  }, [isEditing]);

  const handleDelete = () => {
    setShowDeleteDialog(true);
  };

  const confirmDelete = () => {
    removeGame(game);
    setShowDeleteDialog(false);
  };

  return (
    <>
      <Card className="border border-border/40 shadow-sm hover:border-border transition-all duration-200">
        <CardContent className="p-0">
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
        </CardContent>
      </Card>

      {isEditing && (
        <div
          className={`fixed top-0 left-0 right-0 z-50 mx-auto max-w-md p-4 transition-opacity duration-300 ${
            isPopupVisible ? "opacity-100" : "opacity-0"
          }`}
        >
          <div
            className="fixed inset-0 bg-black/50 z-40 transition-opacity duration-300"
            style={{ opacity: isPopupVisible ? 1 : 0 }}
            onClick={() => startEditGame("", fps)}
          />

          <Card className="border border-primary shadow-lg z-50 relative">
            <CardContent className="p-4 space-y-3">
              <div className="flex items-center gap-2 w-full">
                <Gamepad className="h-5 w-5 text-primary" />
                <span className="font-mono text-sm sm:text-base w-full break-words font-medium">
                  {game}
                </span>
              </div>
              <Input
                ref={inputRef}
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
            </CardContent>
          </Card>
        </div>
      )}

      <DeleteGameDialog
        isOpen={showDeleteDialog}
        onClose={() => setShowDeleteDialog(false)}
        onConfirm={confirmDelete}
        gameName={game}
      />
    </>
  );
}

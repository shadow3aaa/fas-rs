import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { FpsValue } from "@/types/config";
import { Pencil, Save, Trash2, X } from "lucide-react";

interface GameItemProps {
  game: string;
  fps: FpsValue;
  editingGame: string | null;
  editingGameFps: string;
  setEditingGameFps: (value: string) => void;
  startEditGame: (game: string, fps: FpsValue) => void;
  saveEditedGame: () => void;
  removeGame: (game: string) => void;
}

export function GameItem({
  game,
  fps,
  editingGame,
  editingGameFps,
  setEditingGameFps,
  startEditGame,
  saveEditedGame,
  removeGame
}: GameItemProps) {
  const isEditing = editingGame === game;

  return (
    <Card key={game} className="border border-gray-200 dark:border-gray-800 max-w-full">
      <CardContent className="py-4 px-3 sm:px-5 flex flex-col items-start gap-3">
        {isEditing ? (
          <div className="flex flex-col items-start w-full">
            <span className="font-mono text-sm sm:text-base w-full break-words">
              &quot;{game}&quot; {" = "}
            </span>
            <Input
              type="text"
              value={editingGameFps}
              onChange={(e) => setEditingGameFps(e.target.value)}
              className="w-full text-sm sm:text-base mb-3"
              placeholder="Enter FPS"
            />
            <div className="flex space-x-3 w-full">
              <Button
                onClick={() => startEditGame("", fps)}
                variant="outline"
                size="sm"
                className="h-10 w-10"
              >
                <X className="h-5 w-5" />
              </Button>
              <Button
                onClick={saveEditedGame}
                size="sm"
                className="h-10 w-10"
              >
                <Save className="h-5 w-5" />
              </Button>
            </div>
          </div>
        ) : (
          <>
            <div className="flex flex-col items-start w-full">
              <span className="font-mono text-sm sm:text-base break-words w-full">
                &quot;{game}&quot; {" = "}
                <span className="text-blue-600 dark:text-blue-400">
                  {Array.isArray(fps) ? `[${fps.join(', ')}]` : fps}
                </span>
              </span>
            </div>
            <div className="flex space-x-3 w-full mt-2">
              <Button
                onClick={() => startEditGame(game, fps)}
                variant="outline"
                size="sm"
                className="h-10 w-10"
              >
                <Pencil className="h-5 w-5" />
              </Button>
              <Button
                onClick={() => removeGame(game)}
                variant="destructive"
                size="sm"
                className="h-10 w-10"
              >
                <Trash2 className="h-5 w-5" />
              </Button>
            </div>
          </>
        )}
      </CardContent>
    </Card>
  );
}

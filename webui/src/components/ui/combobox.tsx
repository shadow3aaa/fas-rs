"use client";

import * as React from "react";
import { Check, ChevronsUpDown } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";

interface ComboboxProps {
  value: string;
  onValueChange: (value: string) => void;
  options: Array<{ label: string; value: string; disabled?: boolean }>;
  placeholder?: string;
  emptyText?: string;
  searchText?: string;
  className?: string;
}

export function Combobox({
  value,
  onValueChange,
  options,
  placeholder,
  emptyText,
  searchText,
  className,
}: ComboboxProps) {
  const [open, setOpen] = React.useState(false);

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          role="combobox"
          aria-expanded={open}
          className={cn(
            "w-full justify-between h-10 px-3 text-left font-normal bg-card border-neutral-700",
            className,
          )}
        >
          <span className="truncate">
            {value
              ? options.find((option) => option.value === value)?.label
              : placeholder}
          </span>
          <ChevronsUpDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
        </Button>
      </PopoverTrigger>
      <PopoverContent
        className="w-[var(--radix-popover-trigger-width)] p-0 max-h-[250px]"
        align="start"
        sideOffset={4}
      >
        <Command className="max-h-[250px]">
          <CommandInput placeholder={searchText} className="h-8" />
          <CommandList className="max-h-[200px]">
            <CommandEmpty className="py-2 text-center text-sm">
              {emptyText}
            </CommandEmpty>
            <CommandGroup className="overflow-y-auto">
              {options.map((option) => (
                <CommandItem
                  key={option.value}
                  value={option.value}
                  onSelect={(currentValue) => {
                    if (!option.disabled) {
                      onValueChange(currentValue === value ? "" : currentValue);
                      setOpen(false);
                    }
                  }}
                  disabled={option.disabled}
                  className={cn(
                    "py-1.5",
                    option.disabled && "opacity-50 cursor-not-allowed"
                  )}
                >
                  <span className="flex-1 truncate">{option.label}</span>
                  <Check
                    className={cn(
                      "ml-2 h-4 w-4 shrink-0",
                      value === option.value ? "opacity-100" : "opacity-0",
                    )}
                  />
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}

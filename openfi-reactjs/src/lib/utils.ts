import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function formatSeconds(totalSeconds: number, maxUnits = 2) {
  if (!totalSeconds || totalSeconds <= 0) return '0 seconds';

  const timeUnits = [
    { label: 'year', seconds: 31536000 }, 
    { label: 'month', seconds: 2592000 }, 
    { label: 'day', seconds: 86400 },
    { label: 'hour', seconds: 3600 },
    { label: 'minute', seconds: 60 },
    { label: 'second', seconds: 1 }
  ];

  const parts = [];
  let remainingSeconds = totalSeconds;

  for (const unit of timeUnits) {
    if (remainingSeconds >= unit.seconds) {
      const count = Math.floor(remainingSeconds / unit.seconds);
      remainingSeconds %= unit.seconds; // pass the remainder to the next loop
      
      parts.push(`${count} ${unit.label}${count !== 1 ? 's' : ''}`);
    }
  }

  const displayParts = maxUnits ? parts.slice(0, maxUnits) : parts;

  if (displayParts.length === 0) return '0 seconds';
  if (displayParts.length === 1) return displayParts[0];
  if (displayParts.length === 2) return `${displayParts[0]} and ${displayParts[1]}`;
  
  const lastPart = displayParts.pop();
  return `${displayParts.join(', ')} and ${lastPart}`;
}
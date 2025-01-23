// src/lib/utils.ts

import { type ClassValue, clsx } from 'clsx'
import { UtensilsIcon } from 'lucide-vue-next'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

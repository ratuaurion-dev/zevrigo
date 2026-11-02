'use client';

/**
 * Titik impor tunggal komponen UI untuk halaman explorer. Semua komponen di sini adalah komponen shadcn/ui di folder
 * ini (radix-ui + Tailwind CSS). `Tooltip` dan `Progress` membungkus komponen shadcn dengan API ringkas untuk halaman:
 * tooltip berisi teks, dan progres dalam basis poin bilangan bulat (`bigint`), bukan pecahan.
 */

import * as React from 'react';
import { cn } from '@/lib/utils';
import { Progress as ProgressBar } from './progress';
import { Tooltip as TooltipRoot, TooltipContent, TooltipTrigger } from './tooltip';

export { Alert } from './alert';
export { Button, type ButtonProps } from './button';
export { Card, CardContent, CardDescription, CardHeader, CardTitle } from './card';
export { Input } from './input';
export { Skeleton } from './skeleton';
export { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from './table';

/** Persen bulat 0..100 dari basis poin 0..10000; nilai bukan nol paling sedikit 1 agar tetap terlihat. */
export function basisPointsToPercent(basisPoints: bigint, nonZero = false): number {
  const clamped = basisPoints < 0n ? 0n : basisPoints > 10_000n ? 10_000n : basisPoints;
  if (clamped === 0n) {
    return nonZero ? 1 : 0;
  }
  if (clamped < 100n) {
    return 1;
  }
  return Number(clamped / 100n);
}

export function Progress({
  basisPoints,
  nonZero = false,
  className,
  indicatorClassName,
  label,
}: {
  basisPoints: bigint;
  nonZero?: boolean;
  className?: string;
  indicatorClassName?: string;
  label?: string;
}) {
  return (
    <ProgressBar
      value={basisPointsToPercent(basisPoints, nonZero)}
      aria-label={label}
      className={className}
      indicatorClassName={indicatorClassName}
    />
  );
}

export function Tooltip({
  content,
  children,
  className,
}: {
  content: React.ReactNode;
  children: React.ReactNode;
  className?: string;
}) {
  const trigger = (
    <span className={cn('relative inline-flex items-center', className)}>{children}</span>
  );
  if (content === null || content === undefined || content === '') {
    return trigger;
  }
  return (
    <TooltipRoot>
      <TooltipTrigger asChild>{trigger}</TooltipTrigger>
      <TooltipContent>{content}</TooltipContent>
    </TooltipRoot>
  );
}

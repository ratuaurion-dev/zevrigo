import * as React from 'react';
import { cva, type VariantProps } from 'class-variance-authority';
import { cn } from '@/lib/utils';

const alertVariants = cva('relative w-full rounded-lg border p-4 text-xs', {
  variants: {
    variant: {
      default:
        'border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-900 text-slate-800 dark:text-slate-200',
      destructive:
        'border-red-300 dark:border-red-900/80 bg-red-50/90 dark:bg-red-950/50 text-red-900 dark:text-red-200',
      warning:
        'border-amber-300 dark:border-amber-900/80 bg-amber-50/90 dark:bg-amber-950/50 text-amber-900 dark:text-amber-200',
      info: 'border-sky-300 dark:border-sky-900/80 bg-sky-50/90 dark:bg-sky-950/50 text-sky-900 dark:text-sky-200',
      success:
        'border-emerald-300 dark:border-emerald-900/80 bg-emerald-50/90 dark:bg-emerald-950/50 text-emerald-900 dark:text-emerald-200',
    },
  },
  defaultVariants: {
    variant: 'default',
  },
});

function Alert({
  className,
  variant,
  ...props
}: React.ComponentProps<'div'> & VariantProps<typeof alertVariants>) {
  return (
    <div
      data-slot="alert"
      role="alert"
      className={cn(alertVariants({ variant }), className)}
      {...props}
    />
  );
}

export { Alert, alertVariants };

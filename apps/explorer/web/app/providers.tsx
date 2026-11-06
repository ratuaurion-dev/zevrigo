'use client';

import React, { createContext, useCallback, useContext, useEffect, useState } from 'react';
import { Toaster } from 'sonner';
import { TooltipProvider } from '@/components/ui/tooltip';
import {
  detectInitialLang,
  LANG_STORAGE_KEY,
  translate,
  type DictionaryKey,
  type Lang,
} from '@/lib/i18n';

export type Theme = 'light' | 'dark';
const THEME_STORAGE_KEY = 'zvrg-explorer-theme';

interface AppContextValue {
  lang: Lang;
  setLang: (lang: Lang) => void;
  t: (key: DictionaryKey, params?: Record<string, string | number>) => string;
  theme: Theme;
  toggleTheme: () => void;
}

const AppContext = createContext<AppContextValue | null>(null);

function detectInitialTheme(): Theme {
  if (typeof window === 'undefined') return 'dark';
  try {
    const saved = window.localStorage.getItem(THEME_STORAGE_KEY);
    if (saved === 'light' || saved === 'dark') {
      return saved;
    }
  } catch {
    // ignore
  }
  if (
    typeof window.matchMedia === 'function' &&
    window.matchMedia('(prefers-color-scheme: light)').matches
  ) {
    return 'light';
  }
  return 'dark';
}

export function AppProviders({ children }: { children: React.ReactNode }) {
  const [lang, setLangState] = useState<Lang>('id');
  const [theme, setThemeState] = useState<Theme>('dark');

  useEffect(() => {
    const initialLang = detectInitialLang();
    setLangState(initialLang);
    document.documentElement.lang = initialLang;

    const initialTheme = detectInitialTheme();
    setThemeState(initialTheme);
    if (initialTheme === 'dark') {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  }, []);

  const setLang = useCallback((nextLang: Lang) => {
    setLangState(nextLang);
    if (typeof document !== 'undefined') {
      document.documentElement.lang = nextLang;
    }
    try {
      window.localStorage.setItem(LANG_STORAGE_KEY, nextLang);
    } catch {
      // ignore
    }
  }, []);

  const toggleTheme = useCallback(() => {
    setThemeState((prev) => {
      const next: Theme = prev === 'dark' ? 'light' : 'dark';
      if (typeof document !== 'undefined') {
        if (next === 'dark') {
          document.documentElement.classList.add('dark');
        } else {
          document.documentElement.classList.remove('dark');
        }
      }
      try {
        window.localStorage.setItem(THEME_STORAGE_KEY, next);
      } catch {
        // ignore
      }
      return next;
    });
  }, []);

  const t = useCallback(
    (key: DictionaryKey, params?: Record<string, string | number>) => {
      return translate(lang, key, params);
    },
    [lang]
  );

  return (
    <AppContext.Provider value={{ lang, setLang, t, theme, toggleTheme }}>
      <TooltipProvider>{children}</TooltipProvider>
      <Toaster
        position="bottom-right"
        theme={theme}
        toastOptions={{
          className:
            'font-sans text-xs border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 text-slate-900 dark:text-slate-100',
        }}
      />
    </AppContext.Provider>
  );
}

export function useApp(): AppContextValue {
  const ctx = useContext(AppContext);
  if (!ctx) {
    throw new Error('useApp must be used within AppProviders');
  }
  return ctx;
}

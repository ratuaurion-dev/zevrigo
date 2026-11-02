// Bahasa dan teks untuk komponen tampilan (tanpa akses jaringan), serta konteks halaman.
import type { Api } from './api';
import type { Dict, Lang } from './i18n';

export interface Ui {
  t: Dict;
  lang: Lang;
}

export interface Env extends Ui {
  api: Api;
}

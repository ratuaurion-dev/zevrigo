// Daftar blok per 20, terbaru dulu.
import type { BlockRow } from '../api';
import { BlocksTable, Loaded, Pager } from '../components/common';
import { useLoad } from '../load';
import { href } from '../route';
import type { Env, Ui } from '../ui';

export function BlocksView({ rows, before, ui }: { rows: BlockRow[]; before: string | null; ui: Ui }) {
  const { t } = ui;
  const last = rows.length > 0 ? rows[rows.length - 1] : undefined;
  const older = last !== undefined && last.height > 0 ? href({ name: 'blocks', before: String(last.height) }) : null;
  const newer = before === null ? null : href({ name: 'blocks', before: null });
  return (
    <section>
      <h2>{t.navBlocks}</h2>
      <BlocksTable rows={rows} ui={ui} />
      <Pager prev={newer} next={older} prevLabel={t.newer} nextLabel={t.older} />
    </section>
  );
}

export function BlocksPage({ env, before }: { env: Env; before: string | null }) {
  const { load } = useLoad(() => env.api.blocks(before), `blocks:${before ?? ''}`, null);
  return (
    <Loaded load={load} t={env.t}>
      {(data) => <BlocksView rows={data.blocks} before={before} ui={env} />}
    </Loaded>
  );
}

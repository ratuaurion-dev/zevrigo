// Beranda: status sinkronisasi, ringkasan pasokan, dan 20 blok terbaru (diperbarui setiap 10 detik).
import type { Status, SupplyInfo } from '../api';
import { Amount, BlocksTable, Count, Fields, Loaded } from '../components/common';
import { fill } from '../i18n';
import type { Load } from '../load';
import { REFRESH_MS, useLoad } from '../load';
import { href } from '../route';
import type { Env, Ui } from '../ui';

export function StatusPanel({ status, ui }: { status: Status; ui: Ui }) {
  const { t, lang } = ui;
  return (
    <section>
      <h2>{t.statusTitle}</h2>
      <Fields
        rows={[
          [t.network, status.network],
          [t.indexedHeight, status.indexed_height === null ? '—' : <Count value={status.indexed_height} lang={lang} />],
          [t.nodeHeight, status.node_height === null ? '—' : <Count value={status.node_height} lang={lang} />],
          [t.mempoolCount, <a href={href({ name: 'mempool' })}><Count value={status.mempool_count} lang={lang} /></a>],
        ]}
      />
      {status.last_error !== null && (
        <p className="notice notice-error">{fill(t.lastError, { error: status.last_error })}</p>
      )}
    </section>
  );
}

export function SupplySummary({ supply, ui }: { supply: SupplyInfo; ui: Ui }) {
  const { t, lang } = ui;
  return (
    <section>
      <h2>
        <a href={href({ name: 'supply' })}>{t.supplyTitle}</a>
      </h2>
      <Fields
        rows={[
          [t.minedTotal, <Amount quanta={supply.mined_total} lang={lang} />],
          [t.genesisAllocation, <a href={href({ name: 'allocation', page: '0' })}><Amount quanta={supply.genesis_allocation} lang={lang} /></a>],
          [t.totalSupply, <Amount quanta={supply.total_supply} lang={lang} />],
          [t.matchesSchedule, supply.matches_schedule ? t.yes : t.no],
        ]}
      />
    </section>
  );
}

export function HomePage({ env, status }: { env: Env; status: Load<Status> }) {
  const { api, t } = env;
  const supply = useLoad(() => api.supply(), 'supply', REFRESH_MS);
  const blocks = useLoad(() => api.blocks(null), 'blocks', REFRESH_MS);
  return (
    <>
      <Loaded load={status} t={t}>
        {(data) => <StatusPanel status={data} ui={env} />}
      </Loaded>
      <Loaded load={supply.load} t={t}>
        {(data) => <SupplySummary supply={data} ui={env} />}
      </Loaded>
      <section>
        <h2>{t.latestBlocks}</h2>
        <Loaded load={blocks.load} t={t}>
          {(data) => <BlocksTable rows={data.blocks} ui={env} />}
        </Loaded>
        <p>
          <a href={href({ name: 'blocks', before: null })}>{t.allBlocks} →</a>
        </p>
      </section>
    </>
  );
}

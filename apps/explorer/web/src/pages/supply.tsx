// Pasokan hasil tambang dibanding jadwal rumus MOD-04, alokasi genesis, dan total beredar.
import type { SupplyInfo } from '../api';
import { Amount, Count, Fields, Loaded } from '../components/common';
import { fill } from '../i18n';
import { useLoad } from '../load';
import { href } from '../route';
import type { Env, Ui } from '../ui';

export function SupplyView({ supply, ui }: { supply: SupplyInfo; ui: Ui }) {
  const { t, lang } = ui;
  return (
    <section>
      <h2>{t.supplyTitle}</h2>
      {!supply.matches_schedule && <p className="notice notice-error">{t.mismatch}</p>}
      <Fields
        rows={[
          [t.indexedHeight, <Count value={supply.height} lang={lang} />],
          [t.minedTotal, <Amount quanta={supply.mined_total} lang={lang} />],
          [t.scheduledMinedTotal, <Amount quanta={supply.scheduled_mined_total} lang={lang} />],
          [t.matchesSchedule, supply.matches_schedule ? t.yes : t.no],
          [t.genesisAllocation, <a href={href({ name: 'allocation', page: '0' })}><Amount quanta={supply.genesis_allocation} lang={lang} /></a>],
          [t.totalSupply, <Amount quanta={supply.total_supply} lang={lang} />],
          [t.feesTotal, <Amount quanta={supply.fees_total} lang={lang} />],
          [t.nextSubsidy, <Amount quanta={supply.next_subsidy} lang={lang} />],
          [t.tipTarget, <code>{supply.tip_target}</code>],
          [
            t.averageBlock,
            supply.average_block_seconds === null ? '—' : fill(t.seconds, { n: String(supply.average_block_seconds) }),
          ],
        ]}
      />
      <p className="muted">{t.supplyNote}</p>
    </section>
  );
}

export function SupplyPage({ env }: { env: Env }) {
  const { load } = useLoad(() => env.api.supply(), 'supply', null);
  return (
    <Loaded load={load} t={env.t}>
      {(data) => <SupplyView supply={data} ui={env} />}
    </Loaded>
  );
}

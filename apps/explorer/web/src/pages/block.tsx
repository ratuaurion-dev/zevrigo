// Satu blok: header lengkap, subsidi, biaya, konfirmasi, dan daftar transaksi.
import type { BlockDetail } from '../api';
import { Amount, BlockLink, Confirmations, Count, Fields, HashText, Loaded, TxLink } from '../components/common';
import { fill } from '../i18n';
import { useLoad } from '../load';
import { formatUtc } from '../time';
import type { Env, Ui } from '../ui';

export function BlockView({ block, ui }: { block: BlockDetail; ui: Ui }) {
  const { t, lang } = ui;
  return (
    <section>
      <h2>{fill(t.block, { h: String(block.height) })}</h2>
      <Fields
        rows={[
          [t.blockId, <HashText value={block.block_id} />],
          [t.confirmationsHeader, <Confirmations n={block.confirmations} t={t} />],
          [t.time, formatUtc(block.timestamp)],
          [t.prevBlock, block.height === 0 ? '—' : <BlockLink id={block.prev_block_id} />],
          [t.nextBlock, block.next_block_id === null ? '—' : <BlockLink id={block.next_block_id} />],
          [t.merkleRoot, <HashText value={block.merkle_root} />],
          [t.target, <code>{block.target}</code>],
          [t.nonce, <code>{block.nonce}</code>],
          [t.size, <Count value={block.size} lang={lang} />],
          [t.subsidy, <Amount quanta={block.subsidy} lang={lang} />],
          [t.fees, <Amount quanta={block.fees} lang={lang} />],
        ]}
      />
      <p className="muted">{t.confirmationsNote}</p>
      <h3>
        {t.transactions} ({block.tx_count})
      </h3>
      <ol className="list" start={0}>
        {block.txids.map((txid, position) => (
          <li key={txid}>
            <TxLink txid={txid} />
            {position === 0 ? ` (${t.coinbaseLabel})` : ''}
          </li>
        ))}
      </ol>
    </section>
  );
}

export function BlockPage({ env, blockKey }: { env: Env; blockKey: string }) {
  const { load } = useLoad(() => env.api.block(blockKey), `block:${blockKey}`, null);
  return (
    <Loaded load={load} t={env.t}>
      {(data) => <BlockView block={data} ui={env} />}
    </Loaded>
  );
}

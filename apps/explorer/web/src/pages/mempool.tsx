// Mempool: jumlah dan daftar txid (RPC v0.1 belum memberi isi transaksi).
import type { MempoolInfo } from '../api';
import { Count, Loaded, TxLink } from '../components/common';
import { useLoad } from '../load';
import type { Env, Ui } from '../ui';

export function MempoolView({ mempool, ui }: { mempool: MempoolInfo; ui: Ui }) {
  const { t, lang } = ui;
  return (
    <section>
      <h2>
        {t.mempoolTitle} (<Count value={mempool.count} lang={lang} />)
      </h2>
      {mempool.txids.length === 0 ? (
        <p className="muted">{t.mempoolEmpty}</p>
      ) : (
        <ul className="list">
          {mempool.txids.map((txid) => (
            <li key={txid}>
              <TxLink txid={txid} />
            </li>
          ))}
        </ul>
      )}
      <p className="muted">{t.mempoolNote}</p>
    </section>
  );
}

export function MempoolPage({ env }: { env: Env }) {
  const { load, reload } = useLoad(() => env.api.mempool(), 'mempool', null);
  return (
    <>
      <Loaded load={load} t={env.t}>
        {(data) => <MempoolView mempool={data} ui={env} />}
      </Loaded>
      <button type="button" className="button" onClick={reload}>
        {env.t.reload}
      </button>
    </>
  );
}

// Alamat: saldo, total diterima dan dikirim, dan riwayat transaksi per 25.
import type { AddressInfo } from '../api';
import { Amount, Confirmations, Count, Fields, HashText, HeightLink, Loaded, Pager, TxLink } from '../components/common';
import { fill } from '../i18n';
import { useLoad } from '../load';
import { href } from '../route';
import type { Env, Ui } from '../ui';

/** Ringkasan dan riwayat satu alamat. `pageHref` menyusun tautan halaman riwayat lain. */
export function AddressSummary({ info, ui, pageHref }: { info: AddressInfo; ui: Ui; pageHref: (page: number) => string }) {
  const { t, lang } = ui;
  const prev = info.page > 0 ? pageHref(info.page - 1) : null;
  const next = (info.page + 1) * info.page_size < info.tx_count ? pageHref(info.page + 1) : null;
  return (
    <>
      <Fields
        rows={[
          [t.address, <HashText value={info.address} />],
          [t.balance, <Amount quanta={info.balance} lang={lang} />],
          [t.received, <Amount quanta={info.received} lang={lang} />],
          [t.sent, <Amount quanta={info.sent} lang={lang} />],
          [t.utxoCount, <Count value={info.utxo_count} lang={lang} />],
          [t.txCount, <Count value={info.tx_count} lang={lang} />],
        ]}
      />
      <h3>
        {t.history} · {fill(t.page, { n: String(info.page + 1) })}
      </h3>
      {info.txs.length === 0 ? (
        <p className="muted">{t.noHistory}</p>
      ) : (
        <table className="table">
          <thead>
            <tr>
              <th>{t.txid}</th>
              <th>{t.height}</th>
              <th>{t.confirmationsHeader}</th>
              <th>{t.received}</th>
              <th>{t.sent}</th>
            </tr>
          </thead>
          <tbody>
            {info.txs.map((tx) => (
              <tr key={tx.txid}>
                <td>
                  <TxLink txid={tx.txid} short />
                </td>
                <td>
                  <HeightLink height={tx.height} lang={lang} />
                </td>
                <td>
                  <Confirmations n={tx.confirmations} t={t} />
                </td>
                <td>
                  <Amount quanta={tx.received} lang={lang} />
                </td>
                <td>
                  <Amount quanta={tx.sent} lang={lang} />
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      <Pager prev={prev} next={next} prevLabel={t.prevPage} nextLabel={t.nextPage} />
    </>
  );
}

export function AddressView({ info, ui }: { info: AddressInfo; ui: Ui }) {
  return (
    <section>
      <h2>{ui.t.address}</h2>
      <AddressSummary
        info={info}
        ui={ui}
        pageHref={(page) => href({ name: 'address', address: info.address, page: String(page) })}
      />
    </section>
  );
}

export function AddressPage({ env, address, page }: { env: Env; address: string; page: string }) {
  const { load } = useLoad(() => env.api.address(address, page), `address:${address}:${page}`, null);
  return (
    <Loaded load={load} t={env.t}>
      {(data) => <AddressView info={data} ui={env} />}
    </Loaded>
  );
}

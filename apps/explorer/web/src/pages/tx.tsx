// Satu transaksi: blok, posisi, biaya, konfirmasi, input, dan output beserta pembelanjanya.
import type { TxDetail } from '../api';
import {
  AddressLink,
  Amount,
  BlockLink,
  Confirmations,
  Count,
  Fields,
  HashText,
  HeightLink,
  Loaded,
  TxLink,
} from '../components/common';
import { useLoad } from '../load';
import type { Env, Ui } from '../ui';

export function TxView({ tx, ui }: { tx: TxDetail; ui: Ui }) {
  const { t, lang } = ui;
  return (
    <section>
      <h2>{t.transaction}</h2>
      <Fields
        rows={[
          [t.txid, <HashText value={tx.txid} />],
          [t.confirmationsHeader, <Confirmations n={tx.confirmations} t={t} />],
          [t.inBlock, <><HeightLink height={tx.height} lang={lang} /> · <BlockLink id={tx.block_id} short /></>],
          [t.position, String(tx.position)],
          [t.size, <Count value={tx.size} lang={lang} />],
          [t.fee, tx.is_coinbase ? '—' : <Amount quanta={tx.fee} lang={lang} />],
        ]}
      />
      <h3>{t.inputs}</h3>
      {tx.is_coinbase ? (
        <p className="muted">{t.coinbase}</p>
      ) : (
        <table className="table">
          <thead>
            <tr>
              <th>{t.transaction}</th>
              <th>{t.address}</th>
              <th>{t.amount}</th>
            </tr>
          </thead>
          <tbody>
            {tx.inputs.map((input) => (
              <tr key={`${input.txid}:${input.index}`}>
                <td>
                  <TxLink txid={input.txid} short />:{input.index}
                </td>
                <td>
                  <AddressLink address={input.address} />
                </td>
                <td>
                  <Amount quanta={input.amount} lang={lang} />
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      <h3>{t.outputs}</h3>
      <table className="table">
        <thead>
          <tr>
            <th>{t.index}</th>
            <th>{t.address}</th>
            <th>{t.amount}</th>
            <th>{t.spentBy}</th>
          </tr>
        </thead>
        <tbody>
          {tx.outputs.map((output) => (
            <tr key={output.index}>
              <td>{output.index}</td>
              <td>
                <AddressLink address={output.address} />
              </td>
              <td>
                <Amount quanta={output.amount} lang={lang} />
              </td>
              <td>
                {output.spent_by === null ? (
                  <span className="muted">{t.unspent}</span>
                ) : (
                  <>
                    <TxLink txid={output.spent_by.txid} short /> · <HeightLink height={output.spent_by.height} lang={lang} />
                  </>
                )}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}

export function TxPage({ env, txid }: { env: Env; txid: string }) {
  const { load } = useLoad(() => env.api.tx(txid), `tx:${txid}`, null);
  return (
    <Loaded load={load} t={env.t}>
      {(data) => <TxView tx={data} ui={env} />}
    </Loaded>
  );
}

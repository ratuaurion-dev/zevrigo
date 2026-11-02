// Halaman alokasi pencipta (Dokumen Aplikasi Bagian 7): setiap output genesis, pembelanjanya, dan riwayat penerima.
import type { AllocationInfo } from '../api';
import { AddressLink, Amount, BlockLink, Fields, HeightLink, Loaded, TxLink } from '../components/common';
import { useLoad } from '../load';
import { href } from '../route';
import type { Env, Ui } from '../ui';
import { AddressSummary } from './address';

export function AllocationView({ info, ui }: { info: AllocationInfo; ui: Ui }) {
  const { t, lang } = ui;
  return (
    <section>
      <h2>{t.allocationTitle}</h2>
      <p className="muted">{t.allocationNote}</p>
      <Fields
        rows={[
          [t.genesisBlock, <BlockLink id={info.genesis_block_id} />],
          [t.allocated, <Amount quanta={info.allocated} lang={lang} />],
          [t.stillUnspent, <Amount quanta={info.unspent} lang={lang} />],
        ]}
      />
      <h3>{t.allocationOutputs}</h3>
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
          {info.outputs.map((output) => (
            <tr key={`${output.txid}:${output.index}`}>
              <td>
                <TxLink txid={output.txid} short />:{output.index}
              </td>
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
      <h3>{t.recipients}</h3>
      {info.addresses.map((address) => (
        <article className="card" key={address.address}>
          <AddressSummary
            info={address}
            ui={ui}
            pageHref={(page) => href({ name: 'allocation', page: String(page) })}
          />
        </article>
      ))}
    </section>
  );
}

export function AllocationPage({ env, page }: { env: Env; page: string }) {
  const { load } = useLoad(() => env.api.allocation(page), `allocation:${page}`, null);
  return (
    <Loaded load={load} t={env.t}>
      {(data) => <AllocationView info={data} ui={env} />}
    </Loaded>
  );
}

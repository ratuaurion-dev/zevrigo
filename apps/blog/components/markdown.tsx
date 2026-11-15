import React from 'react';
import Link from 'next/link';
import type { Block, Inline } from '@/lib/markdown';

/** Tautan luar dibuka tanpa referrer dan tanpa akses ke jendela asal. */
function InlineLink({ href, text }: { href: string; text: string }) {
  const className =
    'text-[#0F766E] dark:text-[#2DD4BF] underline underline-offset-4 decoration-[#0F766E]/40 hover:decoration-[#0F766E] dark:decoration-[#2DD4BF]/40 dark:hover:decoration-[#2DD4BF] break-words';
  if (href.startsWith('https://')) {
    return (
      <a href={href} className={className} rel="noopener noreferrer">
        {text}
      </a>
    );
  }
  return (
    <Link href={href} className={className}>
      {text}
    </Link>
  );
}

export function InlineText({ items }: { items: Inline[] }) {
  return (
    <>
      {items.map((item, index) => {
        switch (item.kind) {
          case 'code':
            return (
              <code
                key={index}
                className="bg-[#F4F3EE] dark:bg-[#18181B] border border-[#E4E4E7] dark:border-[#27272A] px-1.5 py-0.5 font-mono text-[0.85em] text-[#0F766E] dark:text-[#2DD4BF] break-all"
              >
                {item.text}
              </code>
            );
          case 'strong':
            return (
              <strong key={index} className="font-semibold text-[#18181B] dark:text-[#F4F4F5]">
                {item.text}
              </strong>
            );
          case 'link':
            return <InlineLink key={index} href={item.href} text={item.text} />;
          case 'text':
            return <React.Fragment key={index}>{item.text}</React.Fragment>;
        }
      })}
    </>
  );
}

const TEXT = 'text-[16px] md:text-[17px] leading-relaxed text-[#27272A] dark:text-[#D4D4D8]';

/** Badan artikel. Paragraf pertama memakai huruf awal besar seperti desain portal. */
export function Markdown({ blocks }: { blocks: Block[] }) {
  const firstParagraph = blocks.findIndex((block) => block.kind === 'paragraph');
  return (
    <div>
      {blocks.map((block, index) => {
        switch (block.kind) {
          case 'heading':
            return block.level === 2 ? (
              <h2
                key={index}
                id={block.id}
                className="scroll-mt-24 mt-12 mb-4 font-serif text-2xl md:text-[26px] font-semibold tracking-tight text-[#18181B] dark:text-[#F4F4F5] pb-2 border-b border-[#E4E4E7] dark:border-[#27272A]"
              >
                {block.text}
              </h2>
            ) : (
              <h3
                key={index}
                id={block.id}
                className="scroll-mt-24 mt-8 mb-3 font-serif text-xl font-semibold text-[#18181B] dark:text-[#F4F4F5]"
              >
                {block.text}
              </h3>
            );
          case 'paragraph':
            return (
              <p
                key={index}
                className={`my-5 ${TEXT} ${
                  index === firstParagraph
                    ? 'first-letter:float-left first-letter:mr-3 first-letter:mt-1 first-letter:font-serif first-letter:text-5xl first-letter:font-semibold first-letter:leading-none first-letter:text-[#18181B] dark:first-letter:text-[#F4F4F5]'
                    : ''
                }`}
              >
                <InlineText items={block.inline} />
              </p>
            );
          case 'list': {
            const ListTag = block.ordered ? 'ol' : 'ul';
            return (
              <ListTag
                key={index}
                className={`my-6 space-y-2.5 pl-6 ${block.ordered ? 'list-decimal' : 'list-disc'} marker:text-[#0F766E] dark:marker:text-[#2DD4BF] ${TEXT}`}
              >
                {block.items.map((item, itemIndex) => (
                  <li key={itemIndex} className="pl-1">
                    <InlineText items={item} />
                  </li>
                ))}
              </ListTag>
            );
          }
          case 'code':
            return (
              <div
                key={index}
                className="my-8 border border-[#E4E4E7] dark:border-[#27272A] bg-[#18181B] dark:bg-[#0D0D10] text-[#FAF9F6]"
              >
                <div className="border-b border-[#27272A] bg-[#27272A]/60 px-4 py-2 text-xs font-mono text-[#A1A1AA]">
                  {block.language || 'teks'}
                </div>
                <pre className="overflow-x-auto p-5 text-xs md:text-[13px] font-mono leading-relaxed">
                  <code>{block.code}</code>
                </pre>
              </div>
            );
          case 'quote':
            return (
              <blockquote
                key={index}
                className="my-10 border-y border-[#E4E4E7] dark:border-[#27272A] bg-[#F4F3EE]/50 dark:bg-[#121215] py-6 px-6 font-serif text-lg md:text-xl italic leading-relaxed text-[#18181B] dark:text-[#F4F4F5]"
              >
                <InlineText items={block.inline} />
              </blockquote>
            );
          case 'table':
            return (
              <div key={index} className="my-8 overflow-x-auto border border-[#E4E4E7] dark:border-[#27272A]">
                <table className="w-full text-sm text-left">
                  <thead className="bg-[#F4F3EE] dark:bg-[#121215]">
                    <tr>
                      {block.header.map((cell, cellIndex) => (
                        <th
                          key={cellIndex}
                          className="px-4 py-2.5 font-semibold text-[#18181B] dark:text-[#F4F4F5] border-b border-[#E4E4E7] dark:border-[#27272A] whitespace-nowrap"
                        >
                          <InlineText items={cell} />
                        </th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {block.rows.map((row, rowIndex) => (
                      <tr key={rowIndex} className="border-b last:border-b-0 border-[#E4E4E7] dark:border-[#27272A]">
                        {row.map((cell, cellIndex) => (
                          <td key={cellIndex} className="px-4 py-2.5 align-top text-[#27272A] dark:text-[#D4D4D8] tabular-nums">
                            <InlineText items={cell} />
                          </td>
                        ))}
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            );
          case 'rule':
            return <hr key={index} className="my-10 border-[#E4E4E7] dark:border-[#27272A]" />;
        }
      })}
    </div>
  );
}

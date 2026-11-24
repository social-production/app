/**
 * A one-page PDF holding one JPEG that fills the page, `mm` = [width, height] in millimetres.
 * Enough for the printable QR cards without a PDF library: a catalogue, a page, the image as
 * a DCT stream and a content stream that paints it.
 */
export function jpegPdf(jpeg: Uint8Array, px: number, py: number, mm: number[]): Blob {
  const pt = (v: number): string => ((v / 25.4) * 72).toFixed(2);
  const [w, h] = [pt(mm[0] ?? 210), pt(mm[1] ?? 297)];
  const draw = `q ${w} 0 0 ${h} 0 0 cm /Im0 Do Q`;
  const text = (s: string): Uint8Array => new TextEncoder().encode(s);
  const parts: Uint8Array[] = [];
  const offsets: number[] = [];
  let length = 0;
  const push = (chunk: Uint8Array): void => {
    parts.push(chunk);
    length += chunk.length;
  };
  const object = (n: number, body: Uint8Array[]): void => {
    offsets[n] = length;
    push(text(`${n} 0 obj\n`));
    body.forEach(push);
    push(text('\nendobj\n'));
  };
  push(text('%PDF-1.4\n%âãÏÓ\n'));
  object(1, [text('<< /Type /Catalog /Pages 2 0 R >>')]);
  object(2, [text('<< /Type /Pages /Kids [3 0 R] /Count 1 >>')]);
  object(3, [
    text(
      `<< /Type /Page /Parent 2 0 R /MediaBox [0 0 ${w} ${h}] /Resources << /XObject << /Im0 4 0 R >> >> /Contents 5 0 R >>`,
    ),
  ]);
  object(4, [
    text(
      `<< /Type /XObject /Subtype /Image /Width ${px} /Height ${py} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode /Length ${jpeg.length} >>\nstream\n`,
    ),
    jpeg,
    text('\nendstream'),
  ]);
  object(5, [text(`<< /Length ${draw.length} >>\nstream\n${draw}\nendstream`)]);
  const xref = length;
  const rows = offsets.slice(1).map((o) => `${String(o).padStart(10, '0')} 00000 n \n`);
  push(text(`xref\n0 6\n0000000000 65535 f \n${rows.join('')}`));
  push(text(`trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n${xref}\n%%EOF\n`));
  return new Blob(parts as BlobPart[], { type: 'application/pdf' });
}

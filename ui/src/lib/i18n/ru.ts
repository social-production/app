import { part1 } from './ru/part1.ts';
import { part2 } from './ru/part2.ts';
import { part3 } from './ru/part3.ts';
import { part4 } from './ru/part4.ts';
import { part5 } from './ru/part5.ts';
import { part6 } from './ru/part6.ts';
import { part7 } from './ru/part7.ts';
import { part8 } from './ru/part8.ts';
import { part9 } from './ru/part9.ts';
import { part10 } from './ru/part10.ts';
import { part11 } from './ru/part11.ts';
import { part12 } from './ru/part12.ts';
import { part13 } from './ru/part13.ts';
import { part14 } from './ru/part14.ts';
import { part15 } from './ru/part15.ts';
import { part16 } from './ru/part16.ts';
import { part17 } from './ru/part17.ts';
import { part18 } from './ru/part18.ts';
import { part19 } from './ru/part19.ts';
import { part20 } from './ru/part20.ts';
import { part21 } from './ru/part21.ts';
import { part22 } from './ru/part22.ts';
import { part23 } from './ru/part23.ts';

/**
 * Russian (ADR 0114): English text → Russian, split into parts to keep files short. A plural
 * entry has one, few and many forms (`languages.ts` `Forms`). Drafted by Claude; the owner reads it before it counts as
 * reviewed. Add new text to the last part.
 */
import type { Entry } from './languages.ts';

export const ru: Record<string, Entry> = {
  ...part1,
  ...part2,
  ...part3,
  ...part4,
  ...part5,
  ...part6,
  ...part7,
  ...part8,
  ...part9,
  ...part10,
  ...part11,
  ...part12,
  ...part13,
  ...part14,
  ...part15,
  ...part16,
  ...part17,
  ...part18,
  ...part19,
  ...part20,
  ...part21,
  ...part22,
  ...part23,
};

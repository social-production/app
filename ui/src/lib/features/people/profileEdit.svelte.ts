import { toAvatar, toCover } from '#lib/components/people/avatar.ts';
import type { Avatar, Person } from '#lib/domain/people.ts';
import { explain } from '#lib/services/errors.ts';
import { getPerson, saveProfile } from '#lib/services/people.ts';
import { getCover, setCover } from '#lib/services/profileNotes.ts';
import { viewer } from '#lib/stores/viewer.svelte.ts';

/** An image's bytes as an object URL; the caller revokes the previous one. */
function urlOf(image: Avatar | null): string | null {
  return image ? URL.createObjectURL(new Blob([image.bytes.slice()], { type: image.mime })) : null;
}

/**
 * Your profile edited from the profile itself (Profiles and Comments mockups §2): the cover and
 * the photo change from their camera buttons, and the Edit profile sheet saves name, about and
 * area. Anyone's cover is read here too, so the hero shows it.
 */
export class ProfileEditor {
  cover = $state<string | null>(null);
  saving = $state(false);
  error = $state<string>();
  open = $state(false);

  constructor(
    private readonly agent: string,
    private readonly reload: () => Promise<void>,
  ) {}

  async loadCover(): Promise<void> {
    const found = await getCover(this.agent).catch(() => null);
    this.showCover(found);
  }

  private showCover(image: Avatar | null): void {
    if (this.cover) URL.revokeObjectURL(this.cover);
    this.cover = urlOf(image);
  }

  /** A new cover photo, cropped to 3:1 (ADR 0107). */
  async pickCover(file: File): Promise<void> {
    await this.run(async () => {
      const image = await toCover(file);
      await setCover(image);
      this.showCover(image);
    });
  }

  async removeCover(): Promise<void> {
    await this.run(async () => {
      await setCover(null);
      this.showCover(null);
    });
  }

  /** A new profile photo; the rest of the profile is kept. */
  async pickPhoto(person: Person, file: File): Promise<void> {
    await this.run(async () => {
      const avatar = await toAvatar(file);
      await this.save(person, { avatar });
    });
  }

  /** Save the profile with `changes` over what it holds now. */
  async save(
    person: Person,
    changes: Partial<{ name: string; bio: string; home_area: string; avatar: Avatar | null }>,
  ): Promise<void> {
    const now = (await getPerson(person.agent).catch(() => person)).profile;
    const next = {
      name: (changes.name ?? now?.name ?? '').trim(),
      bio: changes.bio ?? now?.bio ?? '',
      home_area: (changes.home_area ?? now?.home_area ?? '').trim(),
      avatar: changes.avatar === undefined ? (now?.avatar ?? null) : changes.avatar,
    };
    await saveProfile(next);
    viewer.name = next.name;
    await this.reload();
  }

  async run(work: () => Promise<void>): Promise<boolean> {
    this.saving = true;
    this.error = undefined;
    try {
      await work();
      return true;
    } catch (e) {
      this.error = explain(e);
      return false;
    } finally {
      this.saving = false;
    }
  }
}

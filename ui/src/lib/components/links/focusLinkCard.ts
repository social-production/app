import { tick } from 'svelte';

/** web's focusRequest: scroll a link card into view and flash its highlight. */
export async function focusLinkCard(id: string): Promise<void> {
  await tick();
  const card = document.getElementById(`link-request-${id}`);
  card?.scrollIntoView({ behavior: 'smooth', block: 'center' });
  card?.classList.add('request-highlight');
  window.setTimeout(() => card?.classList.remove('request-highlight'), 1800);
}

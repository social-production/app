import { onConnectionChange, type ConnectionState } from '#lib/holo/index.ts';

/** Reactive view of the node connection for any component that shows status. */
export const connection = $state<{ current: ConnectionState }>({
  current: { status: 'connecting' },
});

onConnectionChange((state) => {
  connection.current = state;
});
